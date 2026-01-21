mod analysis;
mod cache;
mod find_references;
mod graph;
mod io;
mod languages;
pub mod timing;

pub use analysis::{cruxlines, cruxlines_from_inputs, OutputRow};
pub use io::CruxlinesError;
pub use find_references::Location;
pub use languages::Ecosystem;

#[doc(hidden)]
pub fn ecosystem_for_path(path: &std::path::Path) -> Option<Ecosystem> {
    languages::language_for_path(path).map(languages::ecosystem_for_language)
}
