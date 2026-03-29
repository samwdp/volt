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
