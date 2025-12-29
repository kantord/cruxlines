use std::path::Path;

pub(crate) mod java;
pub(crate) mod javascript;
pub(crate) mod kotlin;
pub(crate) mod python;
pub(crate) mod rust;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    Java,
    Kotlin,
    Python,
    JavaScript,
    TypeScript,
    TypeScriptReact,
    Rust,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ecosystem {
    Java,
    Python,
    JavaScript,
    Rust,
}

pub(crate) fn language_for_path(path: &Path) -> Option<Language> {
    let ext = path.extension().and_then(|ext| ext.to_str())?;
    if java::EXTENSIONS.contains(&ext) {
        return Some(Language::Java);
    }
    if kotlin::EXTENSIONS.contains(&ext) {
        return Some(Language::Kotlin);
    }
    if python::EXTENSIONS.contains(&ext) {
        return Some(Language::Python);
    }
    if javascript::EXTENSIONS.contains(&ext) {
        return Some(Language::JavaScript);
    }
    if javascript::TYPESCRIPT_EXTENSIONS.contains(&ext) {
        return Some(Language::TypeScript);
    }
    if javascript::TSX_EXTENSIONS.contains(&ext) {
        return Some(Language::TypeScriptReact);
    }
    if rust::EXTENSIONS.contains(&ext) {
        return Some(Language::Rust);
    }
    None
}

pub(crate) fn ecosystem_for_language(language: Language) -> Ecosystem {
    match language {
        Language::Java | Language::Kotlin => Ecosystem::Java,
        Language::Python => Ecosystem::Python,
        Language::JavaScript | Language::TypeScript | Language::TypeScriptReact => {
            Ecosystem::JavaScript
        }
        Language::Rust => Ecosystem::Rust,
    }
}

pub(crate) fn tree_sitter_language(language: Language) -> tree_sitter::Language {
    match language {
        Language::Java => java::language(),
        Language::Kotlin => kotlin::language(),
        Language::Python => python::language(),
        Language::JavaScript => javascript::language(),
        Language::TypeScript => javascript::language_typescript(),
        Language::TypeScriptReact => javascript::language_tsx(),
        Language::Rust => rust::language(),
    }
}

#[cfg(test)]
mod tests {
    use super::{language_for_path, Language};
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
    fn recognizes_jsx_extension() {
        let lang = language_for_path(&PathBuf::from("file.jsx"));
        assert_eq!(lang, Some(Language::JavaScript));
    }

    #[test]
    fn recognizes_typescript_extension() {
        let lang = language_for_path(&PathBuf::from("file.ts"));
        assert_eq!(lang, Some(Language::TypeScript));
    }

    #[test]
    fn recognizes_tsx_extension() {
        let lang = language_for_path(&PathBuf::from("file.tsx"));
        assert_eq!(lang, Some(Language::TypeScriptReact));
    }

    #[test]
    fn recognizes_rust_extension() {
        let lang = language_for_path(&PathBuf::from("file.rs"));
        assert_eq!(lang, Some(Language::Rust));
    }

    #[test]
    fn recognizes_java_extension() {
        let lang = language_for_path(&PathBuf::from("file.java"));
        assert_eq!(lang, Some(Language::Java));
    }

    #[test]
    fn recognizes_kotlin_extension() {
        let lang = language_for_path(&PathBuf::from("file.kt"));
        assert_eq!(lang, Some(Language::Kotlin));
    }

    #[test]
    fn recognizes_kotlin_script_extension() {
        let lang = language_for_path(&PathBuf::from("file.kts"));
        assert_eq!(lang, Some(Language::Kotlin));
    }

    #[test]
    fn ignores_unknown_extensions() {
        let lang = language_for_path(&PathBuf::from("file.txt"));
        assert_eq!(lang, None);
    }

}
