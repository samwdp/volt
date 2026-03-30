use crate::{
    calculator,
    icon_font::symbols::{cod, md},
};
use editor_plugin_api::{
    HoverProviderTopic, PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope,
    PluginPackage, PluginVimMode,
};

pub const HOOK_HOVER_TOGGLE: &str = "ui.hover.toggle";
pub const HOOK_HOVER_FOCUS: &str = "ui.hover.focus";
pub const HOOK_HOVER_NEXT: &str = "ui.hover.next";
pub const HOOK_HOVER_PREVIOUS: &str = "ui.hover.previous";
pub const PROVIDER_TEST_HOVER: &str = "test-hover";
pub const PROVIDER_LSP: &str = "lsp";
pub const PROVIDER_SIGNATURE_HELP: &str = "signature-help";
pub const PROVIDER_DIAGNOSTICS: &str = "diagnostics";
pub const TOGGLE_CHORD: &str = "K";
pub const NEXT_CHORD: &str = "Ctrl+n";
pub const PREVIOUS_CHORD: &str = "Ctrl+p";
pub const LINE_LIMIT: usize = 10;
pub const TOKEN_ICON: &str = md::MD_HELP_CIRCLE_OUTLINE;
pub const SIGNATURE_ICON: &str = md::MD_SIGNATURE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverProviderConfig {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub buffer_kind: Option<String>,
    pub topics: Vec<HoverProviderTopic>,
}

impl HoverProviderConfig {
    pub fn new(id: impl Into<String>, label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            buffer_kind: None,
            topics: Vec::new(),
        }
    }

    pub fn with_buffer_kind(mut self, buffer_kind: impl Into<String>) -> Self {
        self.buffer_kind = Some(buffer_kind.into());
        self
    }

    pub fn with_topics(mut self, topics: Vec<HoverProviderTopic>) -> Self {
        self.topics = topics;
        self
    }
}

pub fn providers() -> Vec<HoverProviderConfig> {
    vec![
        HoverProviderConfig::new(PROVIDER_LSP, "LSP", md::MD_COMMENT_TEXT_OUTLINE),
        HoverProviderConfig::new(PROVIDER_SIGNATURE_HELP, "Signature", SIGNATURE_ICON),
        HoverProviderConfig::new(
            PROVIDER_DIAGNOSTICS,
            "Diagnostics",
            md::MD_ALERT_CIRCLE_OUTLINE,
        ),
        calculator::hover_provider(),
        HoverProviderConfig::new(PROVIDER_TEST_HOVER, "Token", cod::COD_INFO),
    ]
}

/// Returns the metadata for hover commands and keybindings.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "hover",
        true,
        "Cursor-anchored hover overlay with provider ordering.",
    )
    .with_commands(vec![
        hook_command(
            "hover.toggle",
            "Shows or closes the hover overlay at the cursor without focusing it.",
            HOOK_HOVER_TOGGLE,
            None,
        ),
        hook_command(
            "hover.focus",
            "Moves focus from the buffer into the existing hover overlay.",
            HOOK_HOVER_FOCUS,
            None,
        ),
        hook_command(
            "hover.next",
            "Moves to the next hover provider tab.",
            HOOK_HOVER_NEXT,
            None,
        ),
        hook_command(
            "hover.previous",
            "Moves to the previous hover provider tab.",
            HOOK_HOVER_PREVIOUS,
            None,
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new(NEXT_CHORD, "hover.next", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new(
            PREVIOUS_CHORD,
            "hover.previous",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
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
    fn package_exports_hover_commands_and_keybindings() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "hover.toggle")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "hover.focus")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "hover.next")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "hover.previous")
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == NEXT_CHORD)
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == PREVIOUS_CHORD)
        );
        assert_eq!(TOGGLE_CHORD, "K");
    }

    #[test]
    fn provider_order_matches_current_source_of_truth() {
        let providers = providers();
        assert_eq!(providers[0].id, PROVIDER_LSP);
        assert_eq!(providers[1].id, PROVIDER_SIGNATURE_HELP);
        assert_eq!(providers[2].id, PROVIDER_DIAGNOSTICS);
        assert_eq!(providers[3].id, calculator::PROVIDER_CALCULATOR);
        assert_eq!(providers[4].id, PROVIDER_TEST_HOVER);
        assert_eq!(
            providers[3].buffer_kind.as_deref(),
            Some(calculator::CALCULATOR_KIND)
        );
        assert!(!providers[3].topics.is_empty());
    }
}
