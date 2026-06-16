use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// SQL language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("sql", "SQL", &["sql"], &["sql|prettier|--write"])
}

/// Returns the syntax registration for the SQL tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "sql",
        &["sql"],
        "https://github.com/derekstride/tree-sitter-sql.git",
        "tree-sitter-sql",
        "tree_sitter_sql",
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn sql_syntax_language_metadata() {
        let language = syntax_language();
        let grammar = language.grammar().expect("sql grammar missing");

        assert_eq!(language.id(), "sql");
        assert_eq!(language.file_extensions(), ["sql"]);
        assert_eq!(
            grammar.repository_url(),
            "https://github.com/derekstride/tree-sitter-sql.git"
        );
        assert_eq!(grammar.install_dir_name(), "tree-sitter-sql");
        assert_eq!(grammar.symbol_name(), "tree_sitter_sql");
        assert_eq!(grammar.grammar_dir(), Path::new("."));
        assert_eq!(grammar.source_dir(), Path::new("src"));
    }
}
