mod analysis;
mod find_references;
mod graph;
mod io;
mod languages;

pub use analysis::{cruxlines, cruxlines_from_inputs, OutputRow};
pub use io::CruxlinesError;
pub use find_references::Location;
pub use languages::Ecosystem;

#[doc(hidden)]
pub fn is_supported_path(path: &std::path::Path) -> bool {
    languages::language_for_path(path).is_some()
}

#[doc(hidden)]
pub fn ecosystem_for_path(path: &std::path::Path) -> Option<Ecosystem> {
    languages::language_for_path(path).map(languages::ecosystem_for_language)
}
