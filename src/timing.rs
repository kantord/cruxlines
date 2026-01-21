use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static TIMING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initialize timing based on CRUXLINES_TIMING environment variable
pub fn init() {
    if std::env::var("CRUXLINES_TIMING").is_ok() {
        TIMING_ENABLED.store(true, Ordering::Relaxed);
    }
}

/// Check if timing is enabled
pub fn is_enabled() -> bool {
    TIMING_ENABLED.load(Ordering::Relaxed)
}

/// Log a timing message to stderr if timing is enabled
pub fn log(label: &str, duration: std::time::Duration) {
    if is_enabled() {
        eprintln!(
            "[TIMING] {}: {:.3}ms",
            label,
            duration.as_secs_f64() * 1000.0
        );
    }
}

/// Log a timing message with count information
pub fn log_with_count(label: &str, duration: std::time::Duration, count: usize) {
    if is_enabled() {
        eprintln!(
            "[TIMING] {}: {:.3}ms ({} items, {:.3}µs/item)",
            label,
            duration.as_secs_f64() * 1000.0,
            count,
            if count > 0 {
                duration.as_secs_f64() * 1_000_000.0 / count as f64
            } else {
                0.0
            }
        );
    }
}

/// A guard that logs timing when dropped
pub struct TimingGuard {
    label: &'static str,
    start: Instant,
}

impl TimingGuard {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            start: Instant::now(),
        }
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        log(self.label, self.start.elapsed());
    }
}

/// Macro to time a block of code
#[macro_export]
macro_rules! time_block {
    ($label:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        $crate::timing::log($label, start.elapsed());
        result
    }};
}

/// Macro to time a block with count
#[macro_export]
macro_rules! time_block_with_count {
    ($label:expr, $count:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        $crate::timing::log_with_count($label, start.elapsed(), $count);
        result
    }};
}
