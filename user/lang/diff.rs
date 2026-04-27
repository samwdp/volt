use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

const EXTENSIONS: &[&str] = &["diff", "patch"];

/// Returns the syntax registration for the diff tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "diff",
        EXTENSIONS.iter().copied(),
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-diff.git",
            ".",
            "src",
            "tree-sitter-diff",
            "tree_sitter_diff",
        ),
        [
            CaptureThemeMapping::new("comment", "syntax.comment"),
            CaptureThemeMapping::new("constant", "syntax.constant"),
            CaptureThemeMapping::new("attribute", "syntax.attribute"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("variable.parameter", "syntax.variable.parameter"),
            CaptureThemeMapping::new("string.special.path", "syntax.string.special.path"),
            CaptureThemeMapping::new("number", "syntax.number"),
            CaptureThemeMapping::new("punctuation.special", "syntax.punctuation.special"),
            CaptureThemeMapping::new("label", "syntax.label"),
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("diff.plus", "syntax.diff.plus"),
            CaptureThemeMapping::new("diff.minus", "syntax.diff.minus"),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn diff_syntax_language_metadata() {
        let language = syntax_language();
        let extensions = language
            .file_extensions()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let grammar = language.grammar().expect("grammar metadata missing");

        assert_eq!(language.id(), "diff");
        assert_eq!(extensions, EXTENSIONS);
        assert_eq!(
            grammar.repository_url(),
            "https://github.com/tree-sitter-grammars/tree-sitter-diff.git"
        );
        assert_eq!(grammar.install_dir_name(), "tree-sitter-diff");
        assert_eq!(grammar.symbol_name(), "tree_sitter_diff");
        assert_eq!(grammar.grammar_dir(), Path::new("."));
        assert_eq!(grammar.source_dir(), Path::new("src"));
    }

    #[test]
    fn diff_syntax_language_preserves_diff_capture_theme_tokens() {
        let language = syntax_language();
        let mappings = language
            .capture_mappings()
            .iter()
            .map(|mapping| (mapping.capture_name(), mapping.theme_token()))
            .collect::<Vec<_>>();

        assert!(mappings.contains(&("diff.plus", "syntax.diff.plus")));
        assert!(mappings.contains(&("diff.minus", "syntax.diff.minus")));
        assert!(mappings.contains(&("string.special.path", "syntax.string.special.path",)));
    }
}
