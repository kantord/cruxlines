use std::path::Path;

use crate::find_references::Language;

pub const EXTENSION_LANGUAGE_MAP: &[(&str, Language)] = &[
    ("py", Language::Python),
    ("js", Language::JavaScript),
    ("rs", Language::Rust),
];

pub fn language_for_path(path: &Path) -> Option<Language> {
    let ext = path.extension().and_then(|ext| ext.to_str())?;
    EXTENSION_LANGUAGE_MAP
        .iter()
        .find(|(key, _)| *key == ext)
        .map(|(_, language)| *language)
}

#[cfg(test)]
mod tests {
    use super::language_for_path;
    use crate::find_references::Language;
    use std::path::PathBuf;

    #[test]
    fn recognizes_python_extension() {
        let lang = language_for_path(&PathBuf::from("file.py"));
        assert_eq!(lang, Some(Language::Python));
    }

    #[test]
    fn recognizes_javascript_extension() {
        let lang = language_for_path(&PathBuf::from("file.js"));
        assert_eq!(lang, Some(Language::JavaScript));
    }

    #[test]
    fn ignores_unknown_extensions() {
        let lang = language_for_path(&PathBuf::from("file.txt"));
        assert_eq!(lang, None);
    }

    #[test]
    fn recognizes_rust_extension() {
        let lang = language_for_path(&PathBuf::from("file.rs"));
        assert_eq!(lang, Some(Language::Rust));
    }
}
