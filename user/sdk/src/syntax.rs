use std::path::{Path, PathBuf};

use tree_sitter::Language;

use crate::path::{PathMatcher, normalize_extension};

/// Function pointer that returns a statically linked tree-sitter language handle.
pub type LanguageProvider = fn() -> Language;

/// Maps a tree-sitter capture name to a theme token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureThemeMapping {
    pub(crate) capture_name: String,
    pub(crate) theme_token: String,
}

impl CaptureThemeMapping {
    /// Creates a new capture-to-theme mapping.
    pub fn new(capture_name: impl Into<String>, theme_token: impl Into<String>) -> Self {
        Self {
            capture_name: capture_name.into(),
            theme_token: theme_token.into(),
        }
    }

    /// Returns the capture name.
    pub fn capture_name(&self) -> &str {
        &self.capture_name
    }

    /// Returns the destination theme token.
    pub fn theme_token(&self) -> &str {
        &self.theme_token
    }
}

/// Download/build metadata for one installable tree-sitter grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarSource {
    pub(crate) repository_url: String,
    pub(crate) grammar_dir: PathBuf,
    pub(crate) source_dir: PathBuf,
    pub(crate) install_dir_name: String,
    pub(crate) symbol_name: String,
}

impl GrammarSource {
    /// Creates a new installable grammar source description.
    pub fn new(
        repository_url: impl Into<String>,
        grammar_dir: impl Into<PathBuf>,
        source_dir: impl Into<PathBuf>,
        install_dir_name: impl Into<String>,
        symbol_name: impl Into<String>,
    ) -> Self {
        Self {
            repository_url: repository_url.into(),
            grammar_dir: grammar_dir.into(),
            source_dir: source_dir.into(),
            install_dir_name: install_dir_name.into(),
            symbol_name: symbol_name.into(),
        }
    }

    /// Returns the grammar repository URL.
    pub fn repository_url(&self) -> &str {
        &self.repository_url
    }

    /// Returns the subdirectory within the cloned repository that contains the grammar.
    pub fn grammar_dir(&self) -> &Path {
        &self.grammar_dir
    }

    /// Returns the source directory inside the grammar directory.
    pub fn source_dir(&self) -> &Path {
        &self.source_dir
    }

    /// Returns the stable install directory name used under the configured grammar install root.
    pub fn install_dir_name(&self) -> &str {
        &self.install_dir_name
    }

    /// Returns the exported grammar symbol name.
    pub fn symbol_name(&self) -> &str {
        &self.symbol_name
    }

    /// Returns the legacy installed grammar directory under the configured install root.
    pub fn legacy_install_directory(&self, install_root: &Path) -> PathBuf {
        install_root.join(&self.install_dir_name)
    }

    /// Returns the installed shared library path.
    pub fn installed_library_path(&self, install_root: &Path) -> PathBuf {
        install_root.join(shared_library_file_name(&self.install_dir_name))
    }
}

pub fn shared_library_file_name(install_dir_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("lib{install_dir_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{install_dir_name}.dylib")
    } else {
        format!("lib{install_dir_name}.so")
    }
}

fn normalize_unique_entries<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for value in values {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() || normalized.iter().any(|entry| entry == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_owned());
    }
    normalized
}

#[derive(Debug, Clone)]
pub enum LanguageLoader {
    Static {
        language_provider: LanguageProvider,
        highlight_query: String,
    },
    Grammar {
        grammar: GrammarSource,
    },
}

/// User-facing registration for one syntax language.
#[derive(Debug, Clone)]
pub struct LanguageConfiguration {
    pub(crate) id: String,
    pub(crate) file_extensions: Vec<String>,
    pub(crate) file_names: Vec<String>,
    pub(crate) file_globs: Vec<String>,
    pub(crate) capture_mappings: Vec<CaptureThemeMapping>,
    pub(crate) loader: LanguageLoader,
    pub(crate) extra_highlight_query: Option<String>,
    pub(crate) extra_indent_query: Option<String>,
    pub(crate) extra_injections_query: Option<String>,
    pub(crate) extra_locals_query: Option<String>,
    pub(crate) extra_folds_query: Option<String>,
    pub(crate) additional_highlight_languages: Vec<String>,
    pub(crate) path_matcher: PathMatcher,
}

impl LanguageConfiguration {
    /// Creates a statically linked language configuration.
    pub fn new<I, S>(
        id: impl Into<String>,
        file_extensions: I,
        language_provider: LanguageProvider,
        highlight_query: impl Into<String>,
        capture_mappings: impl IntoIterator<Item = CaptureThemeMapping>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::normalize(
            id,
            file_extensions,
            capture_mappings,
            LanguageLoader::Static {
                language_provider,
                highlight_query: highlight_query.into(),
            },
        )
    }

    /// Creates an installable grammar-backed language configuration.
    pub fn from_grammar<I, S>(
        id: impl Into<String>,
        file_extensions: I,
        grammar: GrammarSource,
        capture_mappings: impl IntoIterator<Item = CaptureThemeMapping>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::normalize(
            id,
            file_extensions,
            capture_mappings,
            LanguageLoader::Grammar { grammar },
        )
    }

    pub(crate) fn normalize<I, S>(
        id: impl Into<String>,
        file_extensions: I,
        capture_mappings: impl IntoIterator<Item = CaptureThemeMapping>,
        loader: LanguageLoader,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut normalized_extensions = Vec::new();
        for extension in file_extensions {
            let extension = normalize_extension(&extension.into());
            if !extension.is_empty() && !normalized_extensions.contains(&extension) {
                normalized_extensions.push(extension);
            }
        }
        let path_matcher =
            PathMatcher::from_parts(&normalized_extensions, [] as [&str; 0], [] as [&str; 0]);

        Self {
            id: id.into(),
            file_extensions: normalized_extensions,
            file_names: Vec::new(),
            file_globs: Vec::new(),
            capture_mappings: capture_mappings.into_iter().collect(),
            loader,
            extra_highlight_query: None,
            extra_indent_query: None,
            extra_injections_query: None,
            extra_locals_query: None,
            extra_folds_query: None,
            additional_highlight_languages: Vec::new(),
            path_matcher,
        }
    }

    /// Returns the stable language identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the registered file extensions without leading dots.
    pub fn file_extensions(&self) -> &[String] {
        &self.file_extensions
    }

    /// Adds exact basenames that should resolve to this language.
    pub fn with_file_names<I, S>(mut self, file_names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_names = normalize_unique_entries(file_names);
        self.rebuild_path_matcher();
        self
    }

    /// Returns the registered exact basenames.
    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }

    /// Adds glob patterns that should resolve to this language.
    pub fn with_file_globs<I, S>(mut self, file_globs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_globs = normalize_unique_entries(file_globs);
        self.rebuild_path_matcher();
        self
    }

    /// Returns the registered basename globs.
    pub fn file_globs(&self) -> &[String] {
        &self.file_globs
    }

    /// Returns the capture-to-theme mappings.
    pub fn capture_mappings(&self) -> &[CaptureThemeMapping] {
        &self.capture_mappings
    }

    /// Returns the inline highlight query for statically linked languages.
    pub fn highlight_query(&self) -> Option<&str> {
        match &self.loader {
            LanguageLoader::Static {
                highlight_query, ..
            } => Some(highlight_query),
            LanguageLoader::Grammar { .. } => None,
        }
    }

    /// Adds an extra highlight query appended at load time.
    pub fn with_extra_highlight_query(mut self, query: impl Into<String>) -> Self {
        self.extra_highlight_query = Some(query.into());
        self
    }

    /// Returns the extra highlight query, when configured.
    pub fn extra_highlight_query(&self) -> Option<&str> {
        self.extra_highlight_query.as_deref()
    }

    /// Adds an extra indent query appended at load time.
    pub fn with_extra_indent_query(mut self, query: impl Into<String>) -> Self {
        self.extra_indent_query = Some(query.into());
        self
    }

    /// Returns the extra indent query, when configured.
    pub fn extra_indent_query(&self) -> Option<&str> {
        self.extra_indent_query.as_deref()
    }

    /// Adds an extra injections query appended at load time.
    pub fn with_extra_injections_query(mut self, query: impl Into<String>) -> Self {
        self.extra_injections_query = Some(query.into());
        self
    }

    /// Returns the extra injections query, when configured.
    pub fn extra_injections_query(&self) -> Option<&str> {
        self.extra_injections_query.as_deref()
    }

    /// Adds an extra locals query appended at load time.
    pub fn with_extra_locals_query(mut self, query: impl Into<String>) -> Self {
        self.extra_locals_query = Some(query.into());
        self
    }

    /// Returns the extra locals query, when configured.
    pub fn extra_locals_query(&self) -> Option<&str> {
        self.extra_locals_query.as_deref()
    }

    /// Adds an extra folds query appended at load time.
    pub fn with_extra_folds_query(mut self, query: impl Into<String>) -> Self {
        self.extra_folds_query = Some(query.into());
        self
    }

    /// Returns the extra folds query, when configured.
    pub fn extra_folds_query(&self) -> Option<&str> {
        self.extra_folds_query.as_deref()
    }

    /// Adds additional language ids to merge highlight spans for this language.
    pub fn with_additional_highlight_languages<I, S>(mut self, languages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut ids = Vec::new();
        for language in languages {
            let language = language.into();
            if !language.is_empty() && !ids.contains(&language) {
                ids.push(language);
            }
        }
        self.additional_highlight_languages = ids;
        self
    }

    /// Returns additional language ids used to merge highlight spans.
    pub fn additional_highlight_languages(&self) -> &[String] {
        &self.additional_highlight_languages
    }

    /// Returns the loader used to obtain a tree-sitter language.
    pub fn loader(&self) -> &LanguageLoader {
        &self.loader
    }

    pub fn path_match_score(&self, path: &Path) -> Option<usize> {
        self.path_matcher.best_match_score(path)
    }

    pub(crate) fn rebuild_path_matcher(&mut self) {
        self.path_matcher =
            PathMatcher::from_parts(&self.file_extensions, &self.file_names, &self.file_globs);
    }

    /// Returns the installable grammar metadata, when present.
    pub fn grammar(&self) -> Option<&GrammarSource> {
        match &self.loader {
            LanguageLoader::Static { .. } => None,
            LanguageLoader::Grammar { grammar } => Some(grammar),
        }
    }
}

/// Line and column pair reported by tree-sitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxPoint {
    /// Zero-based line index.
    pub line: usize,
    /// Zero-based column index in bytes.
    pub column: usize,
}

impl SyntaxPoint {
    /// Creates a new syntax point.
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}
/// One named tree-sitter node in the ancestor chain for a cursor location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNodeContext {
    /// Tree-sitter node kind.
    pub kind: String,
    /// Starting line/column for the node.
    pub start_position: SyntaxPoint,
    /// Exclusive ending line/column for the node.
    pub end_position: SyntaxPoint,
}
