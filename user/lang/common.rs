use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

pub(super) fn package(
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

pub(super) fn syntax_language(
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
