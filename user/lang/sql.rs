use editor_plugin_api::PluginPackage;
use editor_syntax::{Language, LanguageConfiguration};

use super::common;

fn sql_language() -> Language {
    tree_sitter_sequel::LANGUAGE.into()
}

/// SQL language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("sql", "SQL", &["sql"], &["sql|prettier|--write"])
}

/// Returns the syntax registration for the SQL tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::static_syntax_language(
        "sql",
        &["sql"],
        sql_language,
        tree_sitter_sequel::HIGHLIGHTS_QUERY,
    )
}
