use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

/// Returns the syntax registration for the git commit tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "gitcommit",
        ["gitcommit"],
        GrammarSource::new(
            "https://github.com/gbprod/tree-sitter-gitcommit.git",
            ".",
            "src",
            "tree-sitter-gitcommit",
            "tree_sitter_gitcommit",
        ),
        [
            CaptureThemeMapping::new("comment", "syntax.comment"),
            CaptureThemeMapping::new("markup.heading", "syntax.markup.heading"),
            CaptureThemeMapping::new("markup.link", "syntax.markup.link"),
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("string.special.url", "syntax.string.special.url"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("variable.parameter", "syntax.variable.parameter"),
            CaptureThemeMapping::new("punctuation.special", "syntax.punctuation.special"),
            CaptureThemeMapping::new("label", "syntax.label"),
            CaptureThemeMapping::new("comment.error", "syntax.comment.error"),
        ],
    )
}
