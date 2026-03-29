use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

/// Returns plugin packages for the additional bundled languages.
pub fn packages() -> Vec<PluginPackage> {
    vec![
        c_package(),
        cpp_package(),
        css_package(),
        go_package(),
        html_package(),
        make_package(),
        odin_package(),
        python_package(),
        scss_package(),
        sql_package(),
        zig_package(),
    ]
}

/// Returns syntax languages for the additional bundled languages.
pub fn syntax_languages() -> Vec<LanguageConfiguration> {
    vec![
        c_syntax_language(),
        cpp_syntax_language(),
        css_syntax_language(),
        go_syntax_language(),
        html_syntax_language(),
        make_syntax_language(),
        odin_syntax_language(),
        python_syntax_language(),
        scss_syntax_language(),
        sql_syntax_language(),
        zig_syntax_language(),
    ]
}

fn package(
    language_id: &str,
    display_name: &str,
    extensions: &[&str],
    formatters: &[&str],
) -> PluginPackage {
    let package_name = format!("lang-{language_id}");
    let attach_command_name = format!("{package_name}.attach");
    let attached_hook = format!("lang.{language_id}.attached");

    let mut actions = vec![PluginAction::log_message(format!(
        "{display_name} language package attached."
    ))];
    actions.extend(formatters.iter().map(|formatter| {
        PluginAction::emit_hook("workspace.formatter.register", Some(*formatter))
    }));
    actions.push(PluginAction::emit_hook(
        attached_hook.as_str(),
        Some(language_id.to_owned()),
    ));

    let hook_bindings = extensions
        .iter()
        .map(|extension| {
            PluginHookBinding::new(
                "buffer.file-open",
                format!("{package_name}.auto-attach-{}", binding_suffix(extension)),
                attach_command_name.as_str(),
                Some(format!(".{extension}")),
            )
        })
        .collect();

    PluginPackage::new(
        package_name.as_str(),
        true,
        format!("{display_name} language defaults, tree-sitter mapping, and startup hooks."),
    )
    .with_commands(vec![PluginCommand::new(
        attach_command_name.as_str(),
        format!("Attaches {display_name} language defaults to the active workspace."),
        actions,
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        attached_hook.as_str(),
        format!("Runs after the {display_name} language package attaches to a buffer."),
    )])
    .with_hook_bindings(hook_bindings)
}

fn binding_suffix(extension: &str) -> String {
    extension
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn syntax_language(
    language_id: &str,
    extensions: &[&str],
    repository: &str,
    install_dir_name: &str,
    symbol_name: &str,
) -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        language_id,
        extensions.iter().copied(),
        GrammarSource::new(repository, ".", "src", install_dir_name, symbol_name),
        standard_capture_mappings(),
    )
}

fn standard_capture_mappings() -> Vec<CaptureThemeMapping> {
    vec![
        CaptureThemeMapping::new("attribute", "syntax.attribute"),
        CaptureThemeMapping::new("comment", "syntax.comment"),
        CaptureThemeMapping::new("constant", "syntax.constant"),
        CaptureThemeMapping::new("constant.builtin", "syntax.constant.builtin"),
        CaptureThemeMapping::new("constructor", "syntax.constructor"),
        CaptureThemeMapping::new("function", "syntax.function"),
        CaptureThemeMapping::new("function.builtin", "syntax.function.builtin"),
        CaptureThemeMapping::new("function.method", "syntax.function.method"),
        CaptureThemeMapping::new("keyword", "syntax.keyword"),
        CaptureThemeMapping::new("keyword.directive", "syntax.keyword.directive"),
        CaptureThemeMapping::new("label", "syntax.label"),
        CaptureThemeMapping::new("module", "syntax.module"),
        CaptureThemeMapping::new("number", "syntax.number"),
        CaptureThemeMapping::new("operator", "syntax.operator"),
        CaptureThemeMapping::new("property", "syntax.property"),
        CaptureThemeMapping::new("punctuation.bracket", "syntax.punctuation.bracket"),
        CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
        CaptureThemeMapping::new("string", "syntax.string"),
        CaptureThemeMapping::new("string.escape", "syntax.string.escape"),
        CaptureThemeMapping::new("string.special", "syntax.string.special"),
        CaptureThemeMapping::new("tag", "syntax.tag"),
        CaptureThemeMapping::new("tag.attribute", "syntax.tag.attribute"),
        CaptureThemeMapping::new("type", "syntax.type"),
        CaptureThemeMapping::new("type.builtin", "syntax.type.builtin"),
        CaptureThemeMapping::new("variable", "syntax.variable"),
        CaptureThemeMapping::new("variable.builtin", "syntax.variable.builtin"),
    ]
}

fn c_package() -> PluginPackage {
    package("c", "C", &["c", "h"], &["c|clang-format|--style=file|-i"])
}

fn c_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "c",
        &["c", "h"],
        "https://github.com/tree-sitter/tree-sitter-c.git",
        "tree-sitter-c",
        "tree_sitter_c",
    )
}

fn cpp_package() -> PluginPackage {
    package(
        "cpp",
        "C++",
        &["cc", "cpp", "cxx", "hpp", "hh", "hxx"],
        &["cpp|clang-format|--style=file|-i"],
    )
}

fn cpp_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "cpp",
        &["cc", "cpp", "cxx", "hpp", "hh", "hxx"],
        "https://github.com/tree-sitter/tree-sitter-cpp.git",
        "tree-sitter-cpp",
        "tree_sitter_cpp",
    )
}

fn css_package() -> PluginPackage {
    package("css", "CSS", &["css"], &["css|prettier|--write"])
}

fn css_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "css",
        &["css"],
        "https://github.com/tree-sitter/tree-sitter-css.git",
        "tree-sitter-css",
        "tree_sitter_css",
    )
}

fn go_package() -> PluginPackage {
    package("go", "Go", &["go"], &["go|gofmt|-w"])
}

fn go_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "go",
        &["go"],
        "https://github.com/tree-sitter/tree-sitter-go.git",
        "tree-sitter-go",
        "tree_sitter_go",
    )
}

fn html_package() -> PluginPackage {
    package("html", "HTML", &["html", "htm"], &["html|prettier|--write"])
}

fn html_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "html",
        &["html", "htm"],
        "https://github.com/tree-sitter/tree-sitter-html.git",
        "tree-sitter-html",
        "tree_sitter_html",
    )
}

fn make_package() -> PluginPackage {
    package("make", "Make", &["mk", "mak", "make"], &[])
}

fn make_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "make",
        &["mk", "mak", "make"],
        "https://github.com/tree-sitter-grammars/tree-sitter-make.git",
        "tree-sitter-make",
        "tree_sitter_make",
    )
}

fn odin_package() -> PluginPackage {
    package("odin", "Odin", &["odin"], &["odin|odin|fmt"])
}

fn odin_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "odin",
        &["odin"],
        "https://github.com/tree-sitter-grammars/tree-sitter-odin.git",
        "tree-sitter-odin",
        "tree_sitter_odin",
    )
}

fn python_package() -> PluginPackage {
    package("python", "Python", &["py"], &["python|black"])
}

fn python_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "python",
        &["py"],
        "https://github.com/tree-sitter/tree-sitter-python.git",
        "tree-sitter-python",
        "tree_sitter_python",
    )
}

fn scss_package() -> PluginPackage {
    package("scss", "SCSS", &["scss"], &["scss|prettier|--write"])
}

fn scss_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "scss",
        &["scss"],
        "https://github.com/serenadeai/tree-sitter-scss.git",
        "tree-sitter-scss",
        "tree_sitter_scss",
    )
}

fn sql_package() -> PluginPackage {
    package("sql", "SQL", &["sql"], &["sql|prettier|--write"])
}

fn sql_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "sql",
        &["sql"],
        "https://github.com/derekstride/tree-sitter-sql.git",
        "tree-sitter-sql",
        "tree_sitter_sql",
    )
}

fn zig_package() -> PluginPackage {
    package("zig", "Zig", &["zig"], &["zig|zig|fmt"])
}

fn zig_syntax_language() -> LanguageConfiguration {
    syntax_language(
        "zig",
        &["zig"],
        "https://github.com/tree-sitter-grammars/tree-sitter-zig.git",
        "tree-sitter-zig",
        "tree_sitter_zig",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formatter_details(package: &PluginPackage) -> Vec<&str> {
        package
            .commands()
            .iter()
            .flat_map(|command| command.actions())
            .filter_map(|action| action.hook())
            .filter(|hook| hook.hook_name() == "workspace.formatter.register")
            .filter_map(|hook| hook.detail())
            .collect()
    }

    #[test]
    fn packages_register_expected_formatters_and_bindings() {
        let html = html_package();
        assert!(
            html.hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some(".html"))
        );
        assert!(
            html.hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some(".htm"))
        );
        assert_eq!(formatter_details(&html), vec!["html|prettier|--write"]);

        let go = go_package();
        assert_eq!(formatter_details(&go), vec!["go|gofmt|-w"]);

        let make = make_package();
        assert!(formatter_details(&make).is_empty());
        assert!(
            make.hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some(".mk"))
        );
    }

    #[test]
    fn syntax_languages_register_requested_grammars() {
        let languages = syntax_languages();
        assert_eq!(languages.len(), 11);

        let html = languages
            .iter()
            .find(|language| language.id() == "html")
            .expect("html syntax language missing");
        let html_grammar = html.grammar().expect("html grammar missing");
        assert_eq!(html.file_extensions(), ["html", "htm"]);
        assert_eq!(html_grammar.install_dir_name(), "tree-sitter-html");
        assert_eq!(html_grammar.symbol_name(), "tree_sitter_html");

        let scss = languages
            .iter()
            .find(|language| language.id() == "scss")
            .expect("scss syntax language missing");
        let scss_grammar = scss.grammar().expect("scss grammar missing");
        assert_eq!(scss.file_extensions(), ["scss"]);
        assert_eq!(scss_grammar.install_dir_name(), "tree-sitter-scss");
        assert_eq!(scss_grammar.symbol_name(), "tree_sitter_scss");

        let zig = languages
            .iter()
            .find(|language| language.id() == "zig")
            .expect("zig syntax language missing");
        let zig_grammar = zig.grammar().expect("zig grammar missing");
        assert_eq!(zig.file_extensions(), ["zig"]);
        assert_eq!(zig_grammar.install_dir_name(), "tree-sitter-zig");
        assert_eq!(zig_grammar.symbol_name(), "tree_sitter_zig");
    }
}
