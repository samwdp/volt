#![doc = r#"Theme token registration and palette resolution used by UI and syntax layers."#]

use std::{collections::BTreeMap, error::Error, fmt};

pub use editor_plugin_api::{Color, Theme, ThemeOption, ThemeStyle, ThemeTokenStyle};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Theme token registration and palette resolution used by UI and syntax layers.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Errors produced by theme registration or activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeError {
    /// Duplicate theme registration.
    DuplicateTheme(String),
    /// Attempted activation of an unknown theme.
    UnknownTheme(String),
}

impl fmt::Display for ThemeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateTheme(theme_id) => {
                write!(formatter, "theme `{theme_id}` is already registered")
            }
            Self::UnknownTheme(theme_id) => {
                write!(formatter, "theme `{theme_id}` is not registered")
            }
        }
    }
}

impl Error for ThemeError {}

/// Registry of available themes and the current active selection.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ThemeRegistry {
    themes: BTreeMap<String, Theme>,
    active_theme: Option<String>,
}

impl ThemeRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of registered themes.
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    /// Returns whether no themes are registered.
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }

    /// Registers a theme and auto-activates it if none are active.
    pub fn register(&mut self, theme: Theme) -> Result<(), ThemeError> {
        let theme_id = theme.id().to_owned();
        if self.themes.contains_key(&theme_id) {
            return Err(ThemeError::DuplicateTheme(theme_id));
        }
        self.themes.insert(theme_id.clone(), theme);
        if self.active_theme.is_none() {
            self.active_theme = Some(theme_id);
        }
        Ok(())
    }

    /// Registers multiple themes.
    pub fn register_all<I>(&mut self, themes: I) -> Result<(), ThemeError>
    where
        I: IntoIterator<Item = Theme>,
    {
        for theme in themes {
            self.register(theme)?;
        }
        Ok(())
    }

    /// Activates a registered theme.
    pub fn activate(&mut self, theme_id: &str) -> Result<(), ThemeError> {
        if !self.themes.contains_key(theme_id) {
            return Err(ThemeError::UnknownTheme(theme_id.to_owned()));
        }
        self.active_theme = Some(theme_id.to_owned());
        Ok(())
    }

    /// Returns the active theme, if one exists.
    pub fn active_theme(&self) -> Option<&Theme> {
        self.active_theme
            .as_deref()
            .and_then(|theme_id| self.themes.get(theme_id))
    }

    /// Resolves a token color from the active theme.
    pub fn resolve(&self, token: &str) -> Option<Color> {
        self.active_theme().and_then(|theme| theme.color(token))
    }

    /// Resolves a token color and font style from the active theme.
    pub fn resolve_style(&self, token: &str) -> Option<ThemeTokenStyle> {
        self.active_theme()
            .and_then(|theme| theme.token_style(token))
    }

    /// Resolves an option from the active theme.
    pub fn resolve_option(&self, option: &str) -> Option<&ThemeOption> {
        self.active_theme().and_then(|theme| theme.option(option))
    }

    /// Resolves a boolean option from the active theme.
    pub fn resolve_bool(&self, option: &str) -> Option<bool> {
        self.active_theme()
            .and_then(|theme| theme.option_bool(option))
    }

    /// Resolves a numeric option from the active theme.
    pub fn resolve_number(&self, option: &str) -> Option<f64> {
        self.active_theme()
            .and_then(|theme| theme.option_number(option))
    }

    /// Resolves a string option from the active theme.
    pub fn resolve_string(&self, option: &str) -> Option<&str> {
        self.active_theme()
            .and_then(|theme| theme.option_string(option))
    }

    /// Returns all registered themes.
    pub fn themes(&self) -> impl Iterator<Item = &Theme> {
        self.themes.values()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
