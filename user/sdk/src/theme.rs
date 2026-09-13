use std::collections::BTreeMap;

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

/// Font styling applied to a theme token.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ThemeStyle {
    /// Draw token text in bold.
    pub bold: bool,
    /// Draw token text in italic.
    pub italic: bool,
}

impl ThemeStyle {
    /// Creates plain token styling.
    pub const fn plain() -> Self {
        Self {
            bold: false,
            italic: false,
        }
    }

    /// Creates token styling from bold and italic flags.
    pub const fn new(bold: bool, italic: bool) -> Self {
        Self { bold, italic }
    }

    /// Returns whether no extra style is enabled.
    pub const fn is_plain(self) -> bool {
        !self.bold && !self.italic
    }
}

/// Complete color and font styling for a theme token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeTokenStyle {
    /// Foreground color.
    pub color: Color,
    /// Font styling.
    pub style: ThemeStyle,
}

impl ThemeTokenStyle {
    /// Creates token styling from a color and font style flags.
    pub const fn new(color: Color, style: ThemeStyle) -> Self {
        Self { color, style }
    }
}

/// Theme option values parsed from theme definitions.
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeOption {
    /// Boolean option value.
    Bool(bool),
    /// Numeric option value.
    Number(f64),
    /// String option value.
    Text(String),
}

impl ThemeOption {
    /// Returns the option as a boolean value, if it is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the option as a numeric value, if it is one.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the option as a string slice, if it is one.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }
}

impl From<bool> for ThemeOption {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for ThemeOption {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<f32> for ThemeOption {
    fn from(value: f32) -> Self {
        Self::Number(value as f64)
    }
}

impl From<i64> for ThemeOption {
    fn from(value: i64) -> Self {
        Self::Number(value as f64)
    }
}

impl From<u64> for ThemeOption {
    fn from(value: u64) -> Self {
        Self::Number(value as f64)
    }
}

impl From<String> for ThemeOption {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for ThemeOption {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

/// Theme definition registered in Rust code.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    id: String,
    name: String,
    tokens: BTreeMap<String, ThemeTokenStyle>,
    options: BTreeMap<String, ThemeOption>,
}

impl Theme {
    /// Creates a new empty theme.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            tokens: BTreeMap::new(),
            options: BTreeMap::new(),
        }
    }

    /// Adds or replaces a theme token color.
    pub fn with_token(mut self, token: impl Into<String>, color: Color) -> Self {
        self.tokens.insert(
            token.into(),
            ThemeTokenStyle::new(color, ThemeStyle::plain()),
        );
        self
    }

    /// Adds or replaces a theme token color and font style.
    pub fn with_token_style(
        mut self,
        token: impl Into<String>,
        color: Color,
        style: ThemeStyle,
    ) -> Self {
        self.tokens
            .insert(token.into(), ThemeTokenStyle::new(color, style));
        self
    }

    /// Adds or replaces a theme option.
    pub fn with_option(mut self, option: impl Into<String>, value: impl Into<ThemeOption>) -> Self {
        self.options.insert(option.into(), value.into());
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

    /// Returns all registered token styles.
    pub fn tokens(&self) -> &BTreeMap<String, ThemeTokenStyle> {
        &self.tokens
    }

    /// Returns all registered option values.
    pub fn options(&self) -> &BTreeMap<String, ThemeOption> {
        &self.options
    }

    /// Resolves a token color.
    pub fn color(&self, token: &str) -> Option<Color> {
        self.tokens.get(token).map(|style| style.color)
    }

    /// Resolves a token color and font style.
    pub fn token_style(&self, token: &str) -> Option<ThemeTokenStyle> {
        self.tokens.get(token).copied()
    }

    /// Resolves a theme option value.
    pub fn option(&self, option: &str) -> Option<&ThemeOption> {
        self.options.get(option)
    }

    /// Resolves a boolean theme option value.
    pub fn option_bool(&self, option: &str) -> Option<bool> {
        self.option(option).and_then(ThemeOption::as_bool)
    }

    /// Resolves a numeric theme option value.
    pub fn option_number(&self, option: &str) -> Option<f64> {
        self.option(option).and_then(ThemeOption::as_number)
    }

    /// Resolves a string theme option value.
    pub fn option_string(&self, option: &str) -> Option<&str> {
        self.option(option).and_then(ThemeOption::as_str)
    }
}
