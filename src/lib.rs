mod analysis;
mod find_references;
mod graph;
mod languages;

pub use analysis::{cruxlines, cruxlines_profiled, OutputRow, ProfileStats};
pub use find_references::Location;

#[doc(hidden)]
pub fn is_supported_path(path: &std::path::Path) -> bool {
    languages::language_for_path(path).is_some()
}
