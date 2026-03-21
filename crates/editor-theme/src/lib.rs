#![doc = r#"Theme token registration and palette resolution used by UI and syntax layers."#]

use std::{collections::BTreeMap, error::Error, fmt};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Theme token registration and palette resolution used by UI and syntax layers.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// RGBA color used by themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

impl Color {
    /// Creates an opaque RGB color.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Creates an RGBA color.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// How the window background transparency is composited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    /// True window opacity applied via the OS compositor (0.0 = fully
    /// transparent, 1.0 = fully opaque).  This is the default.
    #[default]
    Opacity,
    /// Blurred transparency.  The compositor applies a blur behind the window
    /// in addition to the `opacity` value.  Falls back to plain opacity on
    /// platforms that do not support compositor blur.
    Blur,
}

/// Theme definition registered in Rust code.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    id: String,
    name: String,
    tokens: BTreeMap<String, Color>,
    options: BTreeMap<String, bool>,
    /// Window opacity in the range `[0.0, 1.0]`.  Only the window background
    /// is affected; rendered text is always fully opaque.
    opacity: f32,
    /// Compositing blend mode used together with `opacity`.
    blend_mode: BlendMode,
    /// Optional font family name override (e.g. `"JetBrains Mono"`).
    /// `None` means use the system monospace font discovery fallback.
    font: Option<String>,
    /// Optional font size in pixels.  `None` means inherit from `ShellConfig`.
    font_size: Option<u32>,
}

impl Theme {
    /// Creates a new empty theme with default render settings.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            tokens: BTreeMap::new(),
            options: BTreeMap::new(),
            opacity: 1.0,
            blend_mode: BlendMode::Opacity,
            font: None,
            font_size: None,
        }
    }

    /// Adds or replaces a theme token color.
    pub fn with_token(mut self, token: impl Into<String>, color: Color) -> Self {
        self.tokens.insert(token.into(), color);
        self
    }

    /// Adds or replaces a theme option.
    pub fn with_option(mut self, option: impl Into<String>, value: bool) -> Self {
        self.options.insert(option.into(), value);
        self
    }

    /// Sets the window opacity (clamped to `[0.0, 1.0]`).
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Sets the compositing blend mode.
    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Sets the font family name.
    pub fn with_font(mut self, font: impl Into<String>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the font size in pixels.
    pub fn with_font_size(mut self, font_size: u32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    /// Returns the stable theme identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns all registered token colors.
    pub fn tokens(&self) -> &BTreeMap<String, Color> {
        &self.tokens
    }

    /// Resolves a token color.
    pub fn color(&self, token: &str) -> Option<Color> {
        self.tokens.get(token).copied()
    }

    /// Resolves a theme option value.
    pub fn option(&self, option: &str) -> Option<bool> {
        self.options.get(option).copied()
    }

    /// Returns the window opacity in `[0.0, 1.0]`.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Returns the compositing blend mode.
    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    /// Returns the optional font family name.
    pub fn font(&self) -> Option<&str> {
        self.font.as_deref()
    }

    /// Returns the optional font size in pixels.
    pub fn font_size(&self) -> Option<u32> {
        self.font_size
    }
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

    /// Returns all registered themes in alphabetical order by ID.
    pub fn themes(&self) -> &BTreeMap<String, Theme> {
        &self.themes
    }

    /// Resolves a token from the active theme.
    pub fn resolve(&self, token: &str) -> Option<Color> {
        self.active_theme().and_then(|theme| theme.color(token))
    }

    /// Resolves a boolean option from the active theme.
    pub fn resolve_option(&self, option: &str) -> Option<bool> {
        self.active_theme().and_then(|theme| theme.option(option))
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Theme, ThemeRegistry};

    fn volt_dark() -> Theme {
        Theme::new("volt-dark", "Volt Dark")
            .with_token("syntax.keyword", Color::rgb(198, 120, 221))
            .with_token("syntax.string", Color::rgb(152, 195, 121))
    }

    fn amber() -> Theme {
        Theme::new("amber", "Amber")
            .with_token("syntax.keyword", Color::rgb(255, 191, 105))
            .with_token("syntax.string", Color::rgb(255, 221, 128))
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    #[test]
    fn registry_resolves_tokens_from_active_theme() {
        let mut registry = ThemeRegistry::new();
        must(registry.register_all([volt_dark(), amber()]));
        must(registry.activate("amber"));

        assert_eq!(registry.len(), 2);
        assert_eq!(
            registry.active_theme().map(|theme| theme.id()),
            Some("amber")
        );
        assert_eq!(
            registry.resolve("syntax.keyword"),
            Some(Color::rgb(255, 191, 105))
        );
    }
}
