use crate::icon_font::symbols::{cod, md};
use editor_plugin_api::{
    LspCompletionKind, PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope,
    PluginPackage, PluginVimMode,
};

pub const HOOK_AUTOCOMPLETE_TRIGGER: &str = "ui.autocomplete.trigger";
pub const HOOK_AUTOCOMPLETE_NEXT: &str = "ui.autocomplete.next";
pub const HOOK_AUTOCOMPLETE_PREVIOUS: &str = "ui.autocomplete.previous";
pub const HOOK_AUTOCOMPLETE_ACCEPT: &str = "ui.autocomplete.accept";
pub const HOOK_AUTOCOMPLETE_CANCEL: &str = "ui.autocomplete.cancel";

pub const PROVIDER_BUFFER: &str = "buffer";
pub const PROVIDER_LSP: &str = "lsp";
pub const TRIGGER_CHORD: &str = "Ctrl+Space";
pub const NEXT_CHORD: &str = "Ctrl+n";
pub const PREVIOUS_CHORD: &str = "Ctrl+p";
pub const ACCEPT_CHORD: &str = "Ctrl+y";
pub const RESULT_LIMIT: usize = 8;
pub const PROVIDER_SOURCE_GROUP: &str = "source";
pub const TOKEN_ICON: &str = md::MD_FORM_TEXTBOX;
pub const DOCUMENTATION_ICON: &str = cod::COD_INFO;
pub const BUFFER_ITEM_ICON: &str = cod::COD_TEXT_SIZE;
pub const LSP_ITEM_ICON: &str = cod::COD_SYMBOL_MISC;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutocompleteProviderConfig {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub item_icon: String,
    pub or_group: Option<String>,
}

impl AutocompleteProviderConfig {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: impl Into<String>,
        item_icon: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            item_icon: item_icon.into(),
            or_group: None,
        }
    }

    pub fn with_or_group(mut self, or_group: impl Into<String>) -> Self {
        self.or_group = Some(or_group.into());
        self
    }
}

/// Returns the registered autocomplete backends in priority order.
pub fn backends() -> Vec<AutocompleteProviderConfig> {
    vec![
        AutocompleteProviderConfig::new(
            PROVIDER_LSP,
            "LSP",
            md::MD_COMMENT_TEXT_OUTLINE,
            LSP_ITEM_ICON,
        )
        .with_or_group(PROVIDER_SOURCE_GROUP),
        AutocompleteProviderConfig::new(
            PROVIDER_BUFFER,
            "Buffer",
            cod::COD_TEXT_SIZE,
            BUFFER_ITEM_ICON,
        )
        .with_or_group(PROVIDER_SOURCE_GROUP),
    ]
}

/// Returns the registered autocomplete providers consumed by the shell runtime.
pub fn providers() -> Vec<AutocompleteProviderConfig> {
    backends()
}

pub const fn lsp_kind_icon(kind: Option<LspCompletionKind>) -> &'static str {
    match kind {
        Some(LspCompletionKind::Text) => cod::COD_TEXT_SIZE,
        Some(LspCompletionKind::Method)
        | Some(LspCompletionKind::Function)
        | Some(LspCompletionKind::Constructor) => cod::COD_SYMBOL_METHOD,
        Some(LspCompletionKind::Field) => cod::COD_SYMBOL_FIELD,
        Some(LspCompletionKind::Variable) => cod::COD_SYMBOL_VARIABLE,
        Some(LspCompletionKind::Class) => cod::COD_SYMBOL_CLASS,
        Some(LspCompletionKind::Interface) => cod::COD_SYMBOL_INTERFACE,
        Some(LspCompletionKind::Module) => cod::COD_SYMBOL_NAMESPACE,
        Some(LspCompletionKind::Property) => cod::COD_SYMBOL_PROPERTY,
        Some(LspCompletionKind::Unit) => cod::COD_SYMBOL_RULER,
        Some(LspCompletionKind::Value) => cod::COD_SYMBOL_NUMERIC,
        Some(LspCompletionKind::Enum) => cod::COD_SYMBOL_ENUM,
        Some(LspCompletionKind::Keyword) => cod::COD_SYMBOL_KEYWORD,
        Some(LspCompletionKind::Snippet) => cod::COD_SYMBOL_SNIPPET,
        Some(LspCompletionKind::Color) => cod::COD_SYMBOL_COLOR,
        Some(LspCompletionKind::File) => cod::COD_FILE,
        Some(LspCompletionKind::Reference) => cod::COD_REFERENCES,
        Some(LspCompletionKind::Folder) => cod::COD_FOLDER,
        Some(LspCompletionKind::EnumMember) => cod::COD_SYMBOL_ENUM_MEMBER,
        Some(LspCompletionKind::Constant) => cod::COD_SYMBOL_CONSTANT,
        Some(LspCompletionKind::Struct) => cod::COD_SYMBOL_STRUCTURE,
        Some(LspCompletionKind::Event) => cod::COD_SYMBOL_EVENT,
        Some(LspCompletionKind::Operator) => cod::COD_SYMBOL_OPERATOR,
        Some(LspCompletionKind::TypeParameter) => cod::COD_SYMBOL_PARAMETER,
        None => LSP_ITEM_ICON,
    }
}

/// Returns the metadata for autocomplete commands and keybindings.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "autocomplete",
        true,
        "Provider-backed autocomplete with async buffer suggestions.",
    )
    .with_commands(vec![
        hook_command(
            "autocomplete.trigger",
            "Opens autocomplete for the active insert buffer.",
            HOOK_AUTOCOMPLETE_TRIGGER,
            None,
        ),
        hook_command(
            "autocomplete.next",
            "Moves to the next autocomplete suggestion.",
            HOOK_AUTOCOMPLETE_NEXT,
            None,
        ),
        hook_command(
            "autocomplete.previous",
            "Moves to the previous autocomplete suggestion.",
            HOOK_AUTOCOMPLETE_PREVIOUS,
            None,
        ),
        hook_command(
            "autocomplete.accept",
            "Accepts the selected autocomplete suggestion.",
            HOOK_AUTOCOMPLETE_ACCEPT,
            None,
        ),
        hook_command(
            "autocomplete.cancel",
            "Closes the active autocomplete window.",
            HOOK_AUTOCOMPLETE_CANCEL,
            None,
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new(
            TRIGGER_CHORD,
            "autocomplete.trigger",
            PluginKeymapScope::Global,
        )
        .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new(NEXT_CHORD, "autocomplete.next", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new(
            PREVIOUS_CHORD,
            "autocomplete.previous",
            PluginKeymapScope::Global,
        )
        .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new(
            ACCEPT_CHORD,
            "autocomplete.accept",
            PluginKeymapScope::Global,
        )
        .with_vim_mode(PluginVimMode::Insert),
    ])
}

fn hook_command(
    name: &str,
    description: &str,
    hook_name: &str,
    detail: Option<&str>,
) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, detail)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_commands_and_insert_keybindings() {
        let package = package();
        let command_names = package
            .commands()
            .iter()
            .map(|command| command.name())
            .collect::<Vec<_>>();
        assert!(command_names.contains(&"autocomplete.trigger"));
        assert!(command_names.contains(&"autocomplete.next"));
        assert!(command_names.contains(&"autocomplete.previous"));
        assert!(command_names.contains(&"autocomplete.accept"));

        let key_bindings = package.key_bindings();
        assert!(
            key_bindings
                .iter()
                .any(|binding| binding.chord() == TRIGGER_CHORD)
        );
        assert!(
            key_bindings
                .iter()
                .any(|binding| binding.chord() == ACCEPT_CHORD)
        );
    }

    #[test]
    fn providers_prioritize_lsp_before_buffer() {
        let providers = backends();
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0].id, PROVIDER_LSP);
        assert_eq!(providers[0].label, "LSP");
        assert!(!providers[0].icon.is_empty());
        assert_eq!(
            providers[0].or_group.as_deref(),
            Some(PROVIDER_SOURCE_GROUP)
        );
        assert_eq!(providers[1].id, PROVIDER_BUFFER);
        assert_eq!(providers[1].label, "Buffer");
        assert_eq!(
            providers[1].or_group.as_deref(),
            Some(PROVIDER_SOURCE_GROUP)
        );
    }

    #[test]
    fn lsp_kind_icon_maps_core_symbols() {
        assert_eq!(
            lsp_kind_icon(Some(LspCompletionKind::Function)),
            cod::COD_SYMBOL_METHOD
        );
        assert_eq!(
            lsp_kind_icon(Some(LspCompletionKind::Keyword)),
            cod::COD_SYMBOL_KEYWORD
        );
        assert_eq!(
            lsp_kind_icon(Some(LspCompletionKind::Class)),
            cod::COD_SYMBOL_CLASS
        );
        assert_eq!(lsp_kind_icon(None), LSP_ITEM_ICON);
    }
}
