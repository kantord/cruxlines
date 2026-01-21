use lasso::{Spur, ThreadedRodeo};
use std::path::Path;
use std::sync::LazyLock;

/// Global thread-safe string interner
static INTERNER: LazyLock<ThreadedRodeo> = LazyLock::new(ThreadedRodeo::default);

/// Intern a string, returning a handle
#[inline]
pub fn intern(s: &str) -> Spur {
    INTERNER.get_or_intern(s)
}

/// Intern a path, returning a handle
#[inline]
pub fn intern_path(p: &Path) -> Spur {
    // Use lossy conversion for paths - this is fine for display purposes
    let s = p.to_string_lossy();
    INTERNER.get_or_intern(s.as_ref())
}

/// Resolve an interned handle back to a string
#[inline]
pub fn resolve(key: Spur) -> &'static str {
    INTERNER.resolve(&key)
}

/// Get the interner for batch operations
pub fn interner() -> &'static ThreadedRodeo {
    &INTERNER
}
