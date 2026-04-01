#![doc = r#"Tree-sitter language registration, installation, parsing, and capture-to-theme mapping."#]

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use editor_buffer::{TextBuffer, TextByteChunks, TextEdit, TextPoint};
pub use tree_sitter::Language;
use tree_sitter::{
    InputEdit, Parser, Point, Query, QueryCursor, Range, StreamingIterator, TextProvider, Tree,
};
use tree_sitter_language::LanguageFn;

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Tree-sitter language registration, installation, parsing, and capture-to-theme mapping.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Function pointer that returns a statically linked tree-sitter language handle.
pub type LanguageProvider = fn() -> Language;

/// Maps a tree-sitter capture name to a theme token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureThemeMapping {
    capture_name: String,
    theme_token: String,
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
    repository_url: String,
    grammar_dir: PathBuf,
    source_dir: PathBuf,
    install_dir_name: String,
    symbol_name: String,
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

    /// Returns the installed grammar directory under the configured install root.
    pub fn install_directory(&self, install_root: &Path) -> PathBuf {
        install_root.join(&self.install_dir_name)
    }

    /// Returns the installed highlight query path.
    pub fn installed_highlight_query_path(&self, install_root: &Path) -> PathBuf {
        self.install_directory(install_root)
            .join("queries")
            .join("highlights.scm")
    }

    /// Returns the installed shared library path.
    pub fn installed_library_path(&self, install_root: &Path) -> PathBuf {
        self.install_directory(install_root)
            .join(shared_library_file_name(&self.install_dir_name))
    }
}

#[derive(Debug, Clone)]
enum LanguageLoader {
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
    id: String,
    file_extensions: Vec<String>,
    capture_mappings: Vec<CaptureThemeMapping>,
    loader: LanguageLoader,
    extra_highlight_query: Option<String>,
    additional_highlight_languages: Vec<String>,
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

    fn normalize<I, S>(
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

        Self {
            id: id.into(),
            file_extensions: normalized_extensions,
            capture_mappings: capture_mappings.into_iter().collect(),
            loader,
            extra_highlight_query: None,
            additional_highlight_languages: Vec::new(),
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

/// One highlighted range produced by tree-sitter query captures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    /// Start byte offset.
    pub start_byte: usize,
    /// End byte offset.
    pub end_byte: usize,
    /// Start line/column pair from tree-sitter.
    pub start_position: SyntaxPoint,
    /// End line/column pair from tree-sitter.
    pub end_position: SyntaxPoint,
    /// Original tree-sitter capture name.
    pub capture_name: String,
    /// Resolved theme token.
    pub theme_token: String,
}

/// Syntax parse result for a single buffer snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxSnapshot {
    /// Stable language identifier.
    pub language_id: String,
    /// Root syntax node kind.
    pub root_kind: String,
    /// Whether the parse tree contains errors.
    pub has_errors: bool,
    /// Highlight spans generated from the configured highlight query.
    pub highlight_spans: Vec<HighlightSpan>,
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

impl SyntaxSnapshot {
    /// Returns the number of highlight spans.
    pub fn highlight_count(&self) -> usize {
        self.highlight_spans.len()
    }

    /// Returns highlight spans intersecting a visible line window.
    pub fn visible_spans(&self, start_line: usize, line_count: usize) -> Vec<&HighlightSpan> {
        if line_count == 0 {
            return Vec::new();
        }

        let end_line = start_line + line_count.saturating_sub(1);
        self.highlight_spans
            .iter()
            .filter(|span| {
                span.start_position.line <= end_line && span.end_position.line >= start_line
            })
            .collect()
    }
}

/// Reusable parser/tree state for incremental highlighting of one buffer.
pub struct SyntaxParseSession {
    language_id: String,
    revision: u64,
    parser: Parser,
    tree: Tree,
    last_highlight_window: Option<HighlightWindow>,
    last_snapshot: Option<SyntaxSnapshot>,
}

/// A requested line window for range-limited syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightWindow {
    start_line: usize,
    line_count: usize,
}

impl HighlightWindow {
    /// Creates a new line window.
    pub const fn new(start_line: usize, line_count: usize) -> Self {
        Self {
            start_line,
            line_count,
        }
    }

    /// Returns the first requested line.
    pub const fn start_line(&self) -> usize {
        self.start_line
    }

    /// Returns the requested number of lines.
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    const fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    const fn end_line_exclusive(&self) -> usize {
        self.start_line.saturating_add(self.line_count)
    }
}

/// Errors that can occur while registering, installing, or executing syntax providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxError {
    /// A language id was registered more than once.
    DuplicateLanguageId(String),
    /// A file extension was already assigned to another language.
    DuplicateExtension(String),
    /// A request referenced an unknown file extension.
    UnknownExtension(String),
    /// A request referenced an unknown language id.
    UnknownLanguage(String),
    /// The grammar is not installed at the configured install root.
    GrammarNotInstalled {
        language_id: String,
        install_dir: PathBuf,
    },
    /// A highlight query failed to compile.
    InvalidQuery {
        language_id: String,
        message: String,
    },
    /// The parser failed to accept the requested language.
    ParserConfiguration {
        language_id: String,
        message: String,
    },
    /// The parser did not return a syntax tree.
    ParseCancelled(String),
    /// Included ranges could not be configured for a parser.
    IncludedRangesFailed {
        language_id: String,
        message: String,
    },
    /// File-system work required for installation failed.
    Io {
        operation: String,
        path: PathBuf,
        message: String,
    },
    /// Running an installer command failed.
    InstallCommand {
        language_id: String,
        message: String,
    },
    /// Loading the compiled grammar library failed.
    LibraryLoad {
        language_id: String,
        message: String,
    },
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateLanguageId(language_id) => {
                write!(formatter, "language `{language_id}` is already registered")
            }
            Self::DuplicateExtension(extension) => {
                write!(formatter, "extension `{extension}` is already registered")
            }
            Self::UnknownExtension(extension) => {
                write!(formatter, "no syntax language registered for `{extension}`")
            }
            Self::UnknownLanguage(language_id) => {
                write!(formatter, "language `{language_id}` is not registered")
            }
            Self::GrammarNotInstalled {
                language_id,
                install_dir,
            } => {
                write!(
                    formatter,
                    "grammar `{language_id}` is not installed under `{}`",
                    install_dir.display()
                )
            }
            Self::InvalidQuery {
                language_id,
                message,
            } => {
                write!(
                    formatter,
                    "highlight query for `{language_id}` is invalid: {message}"
                )
            }
            Self::ParserConfiguration {
                language_id,
                message,
            } => {
                write!(
                    formatter,
                    "parser configuration failed for `{language_id}`: {message}"
                )
            }
            Self::ParseCancelled(language_id) => {
                write!(
                    formatter,
                    "parser did not produce a tree for `{language_id}`"
                )
            }
            Self::IncludedRangesFailed {
                language_id,
                message,
            } => {
                write!(
                    formatter,
                    "setting included ranges failed for `{language_id}`: {message}"
                )
            }
            Self::Io {
                operation,
                path,
                message,
            } => {
                write!(
                    formatter,
                    "{operation} failed for `{}`: {message}",
                    path.display()
                )
            }
            Self::InstallCommand {
                language_id,
                message,
            } => {
                write!(formatter, "installing `{language_id}` failed: {message}")
            }
            Self::LibraryLoad {
                language_id,
                message,
            } => {
                write!(formatter, "loading `{language_id}` failed: {message}")
            }
        }
    }
}

impl Error for SyntaxError {}

struct LoadedLanguage {
    _library: Option<libloading::Library>,
    language: Language,
    query: Query,
    capture_mappings: BTreeMap<String, String>,
}

struct ParsedHighlight {
    snapshot: SyntaxSnapshot,
    tree: Tree,
}

struct ParseTreeResult {
    tree: Tree,
    changed_ranges: Option<Vec<Range>>,
    applied_edits: Option<Vec<TextEdit>>,
}

struct TextBufferProvider<'a> {
    buffer: &'a TextBuffer,
}

impl<'a> TextProvider<&'a [u8]> for TextBufferProvider<'a> {
    type I = TextByteChunks<'a>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        self.buffer.byte_slice_chunks(node.byte_range())
    }
}

impl LoadedLanguage {
    fn theme_token_for_capture(&self, capture_name: &str) -> String {
        self.capture_mappings
            .get(capture_name)
            .cloned()
            .unwrap_or_else(|| format!("syntax.{capture_name}"))
    }
}

/// Runtime registry of known tree-sitter languages.
pub struct SyntaxRegistry {
    install_root: PathBuf,
    languages: BTreeMap<String, LanguageConfiguration>,
    extensions: BTreeMap<String, String>,
    loaded: BTreeMap<String, LoadedLanguage>,
}

impl fmt::Debug for SyntaxRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SyntaxRegistry")
            .field("install_root", &self.install_root)
            .field("language_count", &self.languages.len())
            .field("loaded_language_count", &self.loaded.len())
            .finish()
    }
}

impl Default for SyntaxRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxRegistry {
    /// Creates a syntax registry using Volt's default per-user grammar install root.
    pub fn new() -> Self {
        Self::with_install_root(default_install_root())
    }

    /// Creates a syntax registry with an explicit install root.
    pub fn with_install_root(install_root: impl Into<PathBuf>) -> Self {
        Self {
            install_root: install_root.into(),
            languages: BTreeMap::new(),
            extensions: BTreeMap::new(),
            loaded: BTreeMap::new(),
        }
    }

    /// Returns the grammar install root.
    pub fn install_root(&self) -> &Path {
        &self.install_root
    }

    /// Returns the number of registered languages.
    pub fn len(&self) -> usize {
        self.languages.len()
    }

    /// Reports whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.languages.is_empty()
    }

    /// Returns all registered language configurations.
    pub fn languages(&self) -> impl Iterator<Item = &LanguageConfiguration> {
        self.languages.values()
    }

    /// Returns a registered language by identifier.
    pub fn language(&self, language_id: &str) -> Option<&LanguageConfiguration> {
        self.languages.get(language_id)
    }

    /// Registers a single language configuration.
    pub fn register(&mut self, config: LanguageConfiguration) -> Result<(), SyntaxError> {
        let language_id = config.id().to_owned();
        if self.languages.contains_key(&language_id) {
            return Err(SyntaxError::DuplicateLanguageId(language_id));
        }

        for extension in config.file_extensions() {
            if self.extensions.contains_key(extension) {
                return Err(SyntaxError::DuplicateExtension(extension.clone()));
            }
        }

        for extension in config.file_extensions() {
            self.extensions
                .insert(extension.clone(), language_id.clone());
        }
        self.languages.insert(language_id, config);
        Ok(())
    }

    /// Registers multiple language configurations.
    pub fn register_all<I>(&mut self, configs: I) -> Result<(), SyntaxError>
    where
        I: IntoIterator<Item = LanguageConfiguration>,
    {
        for config in configs {
            self.register(config)?;
        }

        Ok(())
    }

    /// Returns the language configuration for an extension, if one exists.
    pub fn language_for_extension(&self, extension: &str) -> Option<&LanguageConfiguration> {
        let extension = normalize_extension(extension);
        let language_id = self.extensions.get(&extension)?;
        self.languages.get(language_id)
    }

    /// Returns the language configuration for a path, if one exists.
    pub fn language_for_path(&self, path: impl AsRef<Path>) -> Option<&LanguageConfiguration> {
        let path = path.as_ref();
        let extension = path.extension()?.to_str()?;
        self.language_for_extension(extension)
    }

    /// Reports whether a grammar-backed language is installed.
    pub fn is_installed(&self, language_id: &str) -> Result<bool, SyntaxError> {
        let Some(config) = self.languages.get(language_id) else {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        };

        Ok(match config.grammar() {
            Some(grammar) => {
                grammar.installed_library_path(&self.install_root).exists()
                    && grammar
                        .installed_highlight_query_path(&self.install_root)
                        .exists()
            }
            None => true,
        })
    }

    /// Installs a grammar-backed language into the configured install root.
    pub fn install_language(&mut self, language_id: &str) -> Result<PathBuf, SyntaxError> {
        let config = self
            .languages
            .get(language_id)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.to_owned()))?;
        let Some(grammar) = config.grammar().cloned() else {
            return Ok(self.install_root.clone());
        };

        let temp_clone_root = std::env::temp_dir().join(format!(
            "volt-treesitter-{}",
            temp_guid_like_directory_name()
        ));
        let _cleanup = TempCloneGuard::new(temp_clone_root.clone());
        let parent = temp_clone_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(std::env::temp_dir);
        fs::create_dir_all(&parent)
            .map_err(|error| io_error("create temp parent", &parent, error))?;

        let clone_output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                grammar.repository_url(),
                &temp_clone_root.display().to_string(),
            ])
            .output()
            .map_err(|error| io_error("run git clone", &temp_clone_root, error))?;
        if !clone_output.status.success() {
            return Err(SyntaxError::InstallCommand {
                language_id: language_id.to_owned(),
                message: command_failure_message("git clone", &clone_output),
            });
        }

        let cloned_grammar_dir = temp_clone_root.join(grammar.grammar_dir());
        if !cloned_grammar_dir.exists() {
            return Err(SyntaxError::Io {
                operation: "locate cloned grammar directory".to_owned(),
                path: cloned_grammar_dir,
                message: "configured grammar directory does not exist in the cloned repository"
                    .to_owned(),
            });
        }

        fs::create_dir_all(&self.install_root)
            .map_err(|error| io_error("create grammar install root", &self.install_root, error))?;
        let install_dir = grammar.install_directory(&self.install_root);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir)
                .map_err(|error| io_error("replace installed grammar", &install_dir, error))?;
        }
        fs::create_dir_all(&install_dir)
            .map_err(|error| io_error("create install directory", &install_dir, error))?;
        let queries_dir = cloned_grammar_dir.join("queries");
        if queries_dir.exists() {
            copy_dir_all(&queries_dir, &install_dir.join("queries"))?;
        }
        ensure_installed_highlight_query_path(
            &config,
            &grammar.installed_highlight_query_path(&self.install_root),
        )?;
        build_shared_library(
            language_id,
            &grammar,
            &cloned_grammar_dir,
            &self.install_root,
        )?;

        self.loaded.remove(language_id);
        Ok(install_dir)
    }

    /// Parses and highlights a buffer for a known file extension.
    pub fn highlight_buffer_for_extension(
        &mut self,
        extension: &str,
        buffer: &TextBuffer,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_extension_impl(extension, buffer, None, None)
    }

    /// Parses and highlights a line window for a known file extension.
    pub fn highlight_buffer_for_extension_window(
        &mut self,
        extension: &str,
        buffer: &TextBuffer,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_extension_impl(extension, buffer, Some(highlight_window), None)
    }

    fn highlight_buffer_for_extension_impl(
        &mut self,
        extension: &str,
        buffer: &TextBuffer,
        highlight_window: Option<HighlightWindow>,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        let extension = normalize_extension(extension);
        let language_id = self
            .extensions
            .get(&extension)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownExtension(extension.clone()))?;
        self.highlight_buffer_for_language_impl(
            &language_id,
            buffer,
            highlight_window,
            parse_session,
        )
    }

    /// Parses and highlights a buffer using a registered language identifier.
    pub fn highlight_buffer_for_language(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, None, None)
    }

    /// Returns named ancestor nodes for a cursor location, ordered innermost to outermost.
    pub fn ancestor_contexts_for_language(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        point: TextPoint,
    ) -> Result<Vec<SyntaxNodeContext>, SyntaxError> {
        let language_id = language_id.to_owned();
        if !self.languages.contains_key(&language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id));
        }
        self.ensure_loaded_language(&language_id)?;
        let loaded = self
            .loaded
            .get(&language_id)
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        let parse_result = parse_tree(&language_id, loaded, buffer, None)?;
        let point = text_point_to_tree_sitter_point(buffer, point);
        let Some(mut node) = parse_result
            .tree
            .root_node()
            .named_descendant_for_point_range(point, point)
        else {
            return Ok(Vec::new());
        };
        let mut contexts = Vec::new();
        loop {
            let start = node.start_position();
            let end = node.end_position();
            // Only keep ancestors whose closing line is at or after the cursor so
            // callers can render closing-line context breadcrumbs.
            if node.is_named() && node.parent().is_some() && end.row >= point.row {
                contexts.push(SyntaxNodeContext {
                    kind: node.kind().to_owned(),
                    start_position: SyntaxPoint::new(start.row, start.column),
                    end_position: SyntaxPoint::new(end.row, end.column),
                });
            }
            let Some(parent) = node.parent() else {
                break;
            };
            node = parent;
        }
        Ok(contexts)
    }

    /// Parses and highlights a line window using a registered language identifier.
    pub fn highlight_buffer_for_language_window(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, Some(highlight_window), None)
    }

    /// Parses and highlights a buffer using a reusable parse session.
    pub fn highlight_buffer_for_language_with_session(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, None, Some(parse_session))
    }

    /// Parses and highlights a line window using a reusable parse session.
    pub fn highlight_buffer_for_language_window_with_session(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        highlight_window: HighlightWindow,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(
            language_id,
            buffer,
            Some(highlight_window),
            Some(parse_session),
        )
    }

    fn highlight_buffer_for_language_impl(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        highlight_window: Option<HighlightWindow>,
        mut parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        let language_id = language_id.to_owned();
        let config = self
            .languages
            .get(&language_id)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        self.ensure_loaded_language(&language_id)?;
        let loaded = self
            .loaded
            .get(&language_id)
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        let inline_language_id = "markdown-inline";
        let needs_inline_ranges = config.id() == "markdown"
            && config
                .additional_highlight_languages()
                .iter()
                .any(|language| language == inline_language_id);

        let base_parse = if needs_inline_ranges {
            Some(highlight_loaded_language_with_tree(
                &language_id,
                loaded,
                buffer,
                highlight_window,
                parse_session.as_deref_mut(),
            )?)
        } else {
            None
        };

        let mut snapshot = if let Some(parse) = &base_parse {
            parse.snapshot.clone()
        } else {
            highlight_loaded_language(
                &language_id,
                loaded,
                buffer,
                highlight_window,
                parse_session,
            )?
        };

        for extra_language_id in config.additional_highlight_languages() {
            self.ensure_loaded_language(extra_language_id)?;
            let loaded = self
                .loaded
                .get(extra_language_id)
                .ok_or_else(|| SyntaxError::UnknownLanguage(extra_language_id.clone()))?;
            let extra_snapshot = if needs_inline_ranges && extra_language_id == inline_language_id {
                let Some(parse) = base_parse.as_ref() else {
                    continue;
                };
                let mut inline_lines = markdown_inline_line_indices(&parse.tree);
                if let Some(highlight_window) = highlight_window {
                    let end_line = highlight_window.end_line_exclusive();
                    inline_lines
                        .retain(|line| *line >= highlight_window.start_line() && *line < end_line);
                }
                if inline_lines.is_empty() {
                    continue;
                }
                highlight_inline_language_per_line(
                    extra_language_id,
                    loaded,
                    buffer,
                    &inline_lines,
                )?
            } else {
                highlight_loaded_language(
                    extra_language_id,
                    loaded,
                    buffer,
                    highlight_window,
                    None,
                )?
            };
            snapshot
                .highlight_spans
                .extend(extra_snapshot.highlight_spans);
            snapshot.has_errors = snapshot.has_errors || extra_snapshot.has_errors;
        }

        Ok(snapshot)
    }

    /// Parses and highlights a buffer using a file path's extension.
    pub fn highlight_buffer_for_path(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &TextBuffer,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_path_impl(path, buffer, None, None)
    }

    /// Parses and highlights a line window using a file path's extension.
    pub fn highlight_buffer_for_path_window(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &TextBuffer,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_path_impl(path, buffer, Some(highlight_window), None)
    }

    fn highlight_buffer_for_path_impl(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &TextBuffer,
        highlight_window: Option<HighlightWindow>,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| SyntaxError::UnknownExtension(path.display().to_string()))?;
        self.highlight_buffer_for_extension_impl(extension, buffer, highlight_window, parse_session)
    }

    fn ensure_loaded_language(&mut self, language_id: &str) -> Result<(), SyntaxError> {
        if self.loaded.contains_key(language_id) {
            return Ok(());
        }

        let config = self
            .languages
            .get(language_id)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.to_owned()))?;
        let loaded = load_language(&config, &self.install_root)?;
        self.loaded.insert(language_id.to_owned(), loaded);
        Ok(())
    }
}

fn load_language(
    config: &LanguageConfiguration,
    install_root: &Path,
) -> Result<LoadedLanguage, SyntaxError> {
    let capture_mappings = config
        .capture_mappings()
        .iter()
        .map(|mapping| {
            (
                mapping.capture_name().to_owned(),
                mapping.theme_token().to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    match &config.loader {
        LanguageLoader::Static {
            language_provider,
            highlight_query,
        } => {
            let language = language_provider();
            let mut query_source = highlight_query.to_owned();
            if let Some(extra_query) = config.extra_highlight_query() {
                if !query_source.ends_with('\n') {
                    query_source.push('\n');
                }
                query_source.push_str(extra_query);
            }
            let query = Query::new(&language, &query_source).map_err(|error| {
                SyntaxError::InvalidQuery {
                    language_id: config.id().to_owned(),
                    message: error.to_string(),
                }
            })?;
            Ok(LoadedLanguage {
                _library: None,
                language,
                query,
                capture_mappings,
            })
        }
        LanguageLoader::Grammar { grammar } => {
            let install_dir = grammar.install_directory(install_root);
            let library_path = grammar.installed_library_path(install_root);
            let query_path = grammar.installed_highlight_query_path(install_root);
            if !library_path.exists() || !query_path.exists() {
                return Err(SyntaxError::GrammarNotInstalled {
                    language_id: config.id().to_owned(),
                    install_dir,
                });
            }

            let mut query_source = fs::read_to_string(&query_path)
                .map_err(|error| io_error("read highlight query", &query_path, error))?;
            if let Some(extra_query) = config.extra_highlight_query() {
                if !query_source.ends_with('\n') {
                    query_source.push('\n');
                }
                query_source.push_str(extra_query);
            }
            let library = unsafe {
                // SAFETY: The library path is chosen by the installer for a tree-sitter grammar
                // compiled from generated parser sources. We keep the `Library` alive for at least
                // as long as the loaded `Language` is cached in `LoadedLanguage`.
                libloading::Library::new(&library_path)
            }
            .map_err(|error| SyntaxError::LibraryLoad {
                language_id: config.id().to_owned(),
                message: error.to_string(),
            })?;
            let symbol_name = format!("{}\0", grammar.symbol_name());
            let symbol = unsafe {
                // SAFETY: The symbol name comes from the language configuration and points to the
                // standard tree-sitter exported language constructor for the compiled grammar.
                library.get::<unsafe extern "C" fn() -> *const ()>(symbol_name.as_bytes())
            }
            .map_err(|error| SyntaxError::LibraryLoad {
                language_id: config.id().to_owned(),
                message: error.to_string(),
            })?;
            let language_fn = unsafe {
                // SAFETY: Tree-sitter generated grammar libraries export functions matching the
                // `LanguageFn` ABI. The symbol was resolved from the configured exported name above.
                LanguageFn::from_raw(*symbol)
            };
            let language = Language::new(language_fn);
            let query = Query::new(&language, &query_source).map_err(|error| {
                SyntaxError::InvalidQuery {
                    language_id: config.id().to_owned(),
                    message: error.to_string(),
                }
            })?;
            Ok(LoadedLanguage {
                _library: Some(library),
                language,
                query,
                capture_mappings,
            })
        }
    }
}

/// Converts an editor [`TextPoint`] (character columns) into a tree-sitter [`Point`]
/// whose columns are measured in UTF-8 bytes.
fn text_point_to_tree_sitter_point(buffer: &TextBuffer, point: TextPoint) -> Point {
    let max_line = buffer.line_count().saturating_sub(1);
    let line = point.line.min(max_line);
    let text = buffer.line(line).unwrap_or_default();
    let column = text.chars().take(point.column).map(char::len_utf8).sum();
    Point { row: line, column }
}

fn create_parser(language_id: &str, loaded: &LoadedLanguage) -> Result<Parser, SyntaxError> {
    let mut parser = Parser::new();
    parser
        .set_language(&loaded.language)
        .map_err(|error| SyntaxError::ParserConfiguration {
            language_id: language_id.to_owned(),
            message: error.to_string(),
        })?;
    Ok(parser)
}

fn text_edit_to_input_edit(edit: TextEdit) -> InputEdit {
    InputEdit {
        start_byte: edit.start_byte,
        old_end_byte: edit.old_end_byte,
        new_end_byte: edit.new_end_byte,
        start_position: Point {
            row: edit.start_position.line,
            column: edit.start_position.column,
        },
        old_end_position: Point {
            row: edit.old_end_position.line,
            column: edit.old_end_position.column,
        },
        new_end_position: Point {
            row: edit.new_end_position.line,
            column: edit.new_end_position.column,
        },
    }
}

fn parse_with_parser(
    language_id: &str,
    parser: &mut Parser,
    buffer: &TextBuffer,
    old_tree: Option<&Tree>,
) -> Result<Tree, SyntaxError> {
    let byte_count = buffer.byte_count();
    parser
        .parse_with_options(
            &mut |byte_offset, _| {
                if byte_offset >= byte_count {
                    return &[][..];
                }
                let Some((chunk, chunk_start_byte)) = buffer.chunk_at_byte(byte_offset) else {
                    return &[][..];
                };
                &chunk.as_bytes()[byte_offset.saturating_sub(chunk_start_byte)..]
            },
            old_tree,
            None,
        )
        .ok_or_else(|| SyntaxError::ParseCancelled(language_id.to_owned()))
}

fn parse_tree(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<ParseTreeResult, SyntaxError> {
    let Some(parse_session) = parse_session else {
        let mut parser = create_parser(language_id, loaded)?;
        return Ok(ParseTreeResult {
            tree: parse_with_parser(language_id, &mut parser, buffer, None)?,
            changed_ranges: None,
            applied_edits: None,
        });
    };

    if let Some(session) = parse_session.as_mut()
        && session.language_id == language_id
    {
        if session.revision == buffer.revision() {
            return Ok(ParseTreeResult {
                tree: session.tree.clone(),
                changed_ranges: Some(Vec::new()),
                applied_edits: Some(Vec::new()),
            });
        }

        let applied_edits = if session.revision < buffer.revision() {
            buffer.edits_since(session.revision)
        } else {
            None
        };
        let edited_tree = applied_edits.as_ref().map(|edits| {
            let mut tree = session.tree.clone();
            for edit in edits {
                tree.edit(&text_edit_to_input_edit(*edit));
            }
            tree
        });
        let new_tree = parse_with_parser(
            language_id,
            &mut session.parser,
            buffer,
            edited_tree.as_ref(),
        )?;
        let changed_ranges = edited_tree
            .as_ref()
            .map(|previous_tree| previous_tree.changed_ranges(&new_tree).collect::<Vec<_>>());
        session.revision = buffer.revision();
        session.tree = new_tree.clone();
        return Ok(ParseTreeResult {
            tree: new_tree,
            changed_ranges,
            applied_edits,
        });
    }

    let mut parser = create_parser(language_id, loaded)?;
    let tree = parse_with_parser(language_id, &mut parser, buffer, None)?;
    *parse_session = Some(SyntaxParseSession {
        language_id: language_id.to_owned(),
        revision: buffer.revision(),
        parser,
        tree: tree.clone(),
        last_highlight_window: None,
        last_snapshot: None,
    });
    Ok(ParseTreeResult {
        tree,
        changed_ranges: None,
        applied_edits: None,
    })
}

fn highlight_tree(
    loaded: &LoadedLanguage,
    tree: &Tree,
    buffer: &TextBuffer,
    highlight_window: Option<HighlightWindow>,
) -> Vec<HighlightSpan> {
    let mut query_cursor = QueryCursor::new();
    if let Some(highlight_window) = highlight_window.filter(|window| !window.is_empty()) {
        query_cursor.set_point_range(
            Point {
                row: highlight_window.start_line(),
                column: 0,
            }..Point {
                row: highlight_window.end_line_exclusive(),
                column: 0,
            },
        );
    }
    let capture_names = loaded.query.capture_names();
    let mut highlight_spans = Vec::new();
    let mut matches = query_cursor.matches(
        &loaded.query,
        tree.root_node(),
        TextBufferProvider { buffer },
    );
    loop {
        matches.advance();
        let Some(query_match) = matches.get() else {
            break;
        };

        for capture in query_match.captures {
            let node = capture.node;
            let start = node.start_position();
            let end = node.end_position();
            let capture_name = capture_names
                .get(capture.index as usize)
                .map(|name| name.to_string())
                .unwrap_or_default();

            highlight_spans.push(HighlightSpan {
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                start_position: SyntaxPoint::new(start.row, start.column),
                end_position: SyntaxPoint::new(end.row, end.column),
                theme_token: loaded.theme_token_for_capture(&capture_name),
                capture_name,
            });
        }
    }
    highlight_spans
}

fn sort_highlight_spans(highlight_spans: &mut [HighlightSpan]) {
    highlight_spans.sort_by(|left, right| {
        (
            left.start_byte,
            left.end_byte,
            left.start_position.line,
            left.start_position.column,
            &left.capture_name,
            &left.theme_token,
        )
            .cmp(&(
                right.start_byte,
                right.end_byte,
                right.start_position.line,
                right.start_position.column,
                &right.capture_name,
                &right.theme_token,
            ))
    });
}

fn span_intersects_window(span: &HighlightSpan, window: HighlightWindow) -> bool {
    span.start_position.line < window.end_line_exclusive()
        && span.end_position.line >= window.start_line()
}

fn apply_text_edits_to_span(mut span: HighlightSpan, edits: &[TextEdit]) -> HighlightSpan {
    for edit in edits {
        let input_edit = text_edit_to_input_edit(*edit);
        let mut range = Range {
            start_byte: span.start_byte,
            end_byte: span.end_byte,
            start_point: Point {
                row: span.start_position.line,
                column: span.start_position.column,
            },
            end_point: Point {
                row: span.end_position.line,
                column: span.end_position.column,
            },
        };
        input_edit.edit_range(&mut range);

        span.start_byte = range.start_byte;
        span.end_byte = range.end_byte;
        span.start_position = SyntaxPoint::new(range.start_point.row, range.start_point.column);
        span.end_position = SyntaxPoint::new(range.end_point.row, range.end_point.column);
    }

    span
}

fn changed_range_windows(
    changed_ranges: &[Range],
    highlight_window: HighlightWindow,
) -> Vec<HighlightWindow> {
    const CONTEXT_LINES: usize = 1;

    if highlight_window.is_empty() {
        return Vec::new();
    }

    let mut ranges = changed_ranges
        .iter()
        .filter_map(|range| {
            let start_line = range
                .start_point
                .row
                .saturating_sub(CONTEXT_LINES)
                .max(highlight_window.start_line());
            let end_line_exclusive = range
                .end_point
                .row
                .saturating_add(1)
                .saturating_add(CONTEXT_LINES)
                .min(highlight_window.end_line_exclusive());
            (start_line < end_line_exclusive)
                .then(|| HighlightWindow::new(start_line, end_line_exclusive - start_line))
        })
        .collect::<Vec<_>>();
    ranges.sort_by_key(HighlightWindow::start_line);

    let mut merged: Vec<HighlightWindow> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && last.end_line_exclusive() >= range.start_line()
        {
            let end_line_exclusive = last.end_line_exclusive().max(range.end_line_exclusive());
            *last = HighlightWindow::new(last.start_line(), end_line_exclusive - last.start_line());
            continue;
        }
        merged.push(range);
    }
    merged
}

fn highlight_loaded_language_with_tree(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    highlight_window: Option<HighlightWindow>,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<ParsedHighlight, SyntaxError> {
    let (parse_result, mut session) = match parse_session {
        Some(parse_session) => (
            parse_tree(language_id, loaded, buffer, Some(parse_session))?,
            parse_session.as_mut(),
        ),
        None => (parse_tree(language_id, loaded, buffer, None)?, None),
    };

    let mut highlight_spans = session
        .as_ref()
        .and_then(|session| {
            let previous_snapshot = session.last_snapshot.as_ref()?;
            if session.last_highlight_window != highlight_window {
                return None;
            }
            let changed_ranges = parse_result.changed_ranges.as_ref()?;
            let applied_edits = parse_result.applied_edits.as_deref().unwrap_or(&[]);
            let previous_highlight_spans = previous_snapshot
                .highlight_spans
                .iter()
                .cloned()
                .map(|span| apply_text_edits_to_span(span, applied_edits))
                .collect::<Vec<_>>();
            if changed_ranges.is_empty() {
                return Some(previous_highlight_spans);
            }
            let highlight_window = highlight_window?;
            let changed_windows = changed_range_windows(changed_ranges, highlight_window);
            if changed_windows.is_empty() {
                return Some(previous_highlight_spans);
            }

            let mut highlight_spans = previous_highlight_spans
                .iter()
                .filter(|span| {
                    !changed_windows
                        .iter()
                        .any(|window| span_intersects_window(span, *window))
                })
                .cloned()
                .collect::<Vec<_>>();
            for changed_window in changed_windows {
                highlight_spans.extend(highlight_tree(
                    loaded,
                    &parse_result.tree,
                    buffer,
                    Some(changed_window),
                ));
            }
            Some(highlight_spans)
        })
        .unwrap_or_else(|| highlight_tree(loaded, &parse_result.tree, buffer, highlight_window));
    sort_highlight_spans(&mut highlight_spans);
    let snapshot = SyntaxSnapshot {
        language_id: language_id.to_owned(),
        root_kind: parse_result.tree.root_node().kind().to_owned(),
        has_errors: parse_result.tree.root_node().has_error(),
        highlight_spans,
    };
    if let Some(session) = session.as_mut() {
        session.last_highlight_window = highlight_window;
        session.last_snapshot = Some(snapshot.clone());
    }
    Ok(ParsedHighlight {
        snapshot,
        tree: parse_result.tree,
    })
}

fn highlight_loaded_language(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    highlight_window: Option<HighlightWindow>,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<SyntaxSnapshot, SyntaxError> {
    Ok(highlight_loaded_language_with_tree(
        language_id,
        loaded,
        buffer,
        highlight_window,
        parse_session,
    )?
    .snapshot)
}

fn markdown_inline_line_indices(tree: &Tree) -> Vec<usize> {
    fn collect_lines(node: tree_sitter::Node<'_>, lines: &mut BTreeSet<usize>) {
        if node.kind() == "inline" {
            for line in node.start_position().row..=node.end_position().row {
                lines.insert(line);
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_lines(child, lines);
        }
    }

    let mut lines = BTreeSet::new();
    collect_lines(tree.root_node(), &mut lines);
    lines.into_iter().collect()
}

fn highlight_inline_language_per_line(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    line_indices: &[usize],
) -> Result<SyntaxSnapshot, SyntaxError> {
    let mut highlight_spans = Vec::new();
    let mut has_errors = false;

    for &line_index in line_indices {
        let Some(line_text) = buffer.line(line_index) else {
            continue;
        };
        if line_text.is_empty() {
            continue;
        }
        let Some(start_byte) = buffer.line_start_byte(line_index) else {
            continue;
        };
        let line_buffer = TextBuffer::from_text(&line_text);
        let mut parser = create_parser(language_id, loaded)?;
        let tree = parse_with_parser(language_id, &mut parser, &line_buffer, None)?;
        has_errors |= tree.root_node().has_error();

        let spans = highlight_tree(loaded, &tree, &line_buffer, None);
        let line_len = line_text.chars().count();
        for span in spans {
            let start_col = span.start_position.column.min(line_len);
            let end_col = span.end_position.column.min(line_len);
            if start_col >= end_col {
                continue;
            }
            highlight_spans.push(HighlightSpan {
                start_byte: start_byte.saturating_add(span.start_byte),
                end_byte: start_byte.saturating_add(span.end_byte),
                start_position: SyntaxPoint::new(line_index, start_col),
                end_position: SyntaxPoint::new(line_index, end_col),
                capture_name: span.capture_name,
                theme_token: span.theme_token,
            });
        }
    }

    Ok(SyntaxSnapshot {
        language_id: language_id.to_owned(),
        root_kind: "inline".to_owned(),
        has_errors,
        highlight_spans,
    })
}

fn build_shared_library(
    language_id: &str,
    grammar: &GrammarSource,
    grammar_dir: &Path,
    install_root: &Path,
) -> Result<(), SyntaxError> {
    let source_dir = grammar_dir.join(grammar.source_dir());
    let parser_path = source_dir.join("parser.c");
    if !parser_path.exists() {
        return Err(SyntaxError::Io {
            operation: "locate parser source".to_owned(),
            path: parser_path,
            message: "parser.c is missing".to_owned(),
        });
    }

    let scanner_c = source_dir.join("scanner.c");
    let scanner_cpp = source_dir.join("scanner.cc");
    let output_path = grammar.installed_library_path(install_root);
    let compiler = if scanner_cpp.exists() { "c++" } else { "cc" };
    let mut command = Command::new(compiler);
    if cfg!(target_os = "macos") {
        command.args(["-fPIC", "-dynamiclib"]);
    } else {
        command.args(["-fPIC", "-shared"]);
    }
    if scanner_cpp.exists() {
        command.arg("-std=c++14");
    }
    command.arg(&parser_path);
    if scanner_c.exists() {
        command.arg(&scanner_c);
    }
    if scanner_cpp.exists() {
        command.arg(&scanner_cpp);
    }
    command.arg("-I");
    command.arg(&source_dir);
    command.arg("-o");
    command.arg(&output_path);
    command.current_dir(grammar_dir);

    let output = command
        .output()
        .map_err(|error| io_error("run grammar compiler", grammar_dir, error))?;
    if !output.status.success() {
        return Err(SyntaxError::InstallCommand {
            language_id: language_id.to_owned(),
            message: command_failure_message(compiler, &output),
        });
    }

    Ok(())
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<(), SyntaxError> {
    fs::create_dir_all(destination)
        .map_err(|error| io_error("create install directory", destination, error))?;

    for entry in
        fs::read_dir(source).map_err(|error| io_error("read source directory", source, error))?
    {
        let entry = entry.map_err(|error| io_error("read directory entry", source, error))?;
        let entry_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = entry
            .metadata()
            .map_err(|error| io_error("read source metadata", &entry_path, error))?;
        if metadata.is_dir() {
            copy_dir_all(&entry_path, &destination_path)?;
        } else {
            fs::copy(&entry_path, &destination_path).map_err(|error| SyntaxError::Io {
                operation: "copy grammar file".to_owned(),
                path: entry_path,
                message: error.to_string(),
            })?;
        }
    }

    Ok(())
}

fn ensure_installed_highlight_query_path(
    config: &LanguageConfiguration,
    query_path: &Path,
) -> Result<(), SyntaxError> {
    if query_path.exists() || config.extra_highlight_query().is_none() {
        return Ok(());
    }
    let parent = query_path.parent().ok_or_else(|| SyntaxError::Io {
        operation: "locate highlight query parent".to_owned(),
        path: query_path.to_path_buf(),
        message: "installed highlight query path has no parent directory".to_owned(),
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| io_error("create highlight query directory", parent, error))?;
    fs::write(query_path, "")
        .map_err(|error| io_error("create placeholder highlight query", query_path, error))
}

fn io_error(operation: &str, path: &Path, error: std::io::Error) -> SyntaxError {
    SyntaxError::Io {
        operation: operation.to_owned(),
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

fn command_failure_message(command_name: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !stdout.is_empty() {
        return stdout;
    }

    format!("{command_name} exited with status {}", output.status)
}

fn default_install_root() -> PathBuf {
    if let Some(path) = env::var_os("VOLT_GRAMMAR_DIR").map(PathBuf::from) {
        return path;
    }

    let base = if cfg!(target_os = "windows") {
        env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from)
    } else {
        env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| {
            env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
    };

    base.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("volt")
        .join("grammars")
}

fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

fn shared_library_file_name(install_dir_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("lib{install_dir_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{install_dir_name}.dylib")
    } else {
        format!("lib{install_dir_name}.so")
    }
}

fn temp_guid_like_directory_name() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let value = duration.as_nanos() ^ ((std::process::id() as u128) << 32);
    let part1 = ((value >> 96) & 0xffff_ffff) as u32;
    let part2 = ((value >> 80) & 0xffff) as u16;
    let part3 = ((value >> 64) & 0xffff) as u16;
    let part4 = ((value >> 48) & 0xffff) as u16;
    let part5 = (value & 0xffff_ffff_ffff) as u64;
    format!("{part1:08x}-{part2:04x}-{part3:04x}-{part4:04x}-{part5:012x}")
}

struct TempCloneGuard {
    path: PathBuf,
}

impl TempCloneGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TempCloneGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        CaptureThemeMapping, GrammarSource, HighlightWindow, LanguageConfiguration, SyntaxError,
        SyntaxParseSession, SyntaxRegistry, ensure_installed_highlight_query_path,
    };
    use editor_buffer::{TextBuffer, TextPoint};

    fn rust_language() -> super::Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn rust_configuration() -> LanguageConfiguration {
        LanguageConfiguration::new(
            "rust",
            ["rs"],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [
                CaptureThemeMapping::new("keyword", "syntax.keyword"),
                CaptureThemeMapping::new("function", "syntax.function"),
                CaptureThemeMapping::new("string", "syntax.string"),
            ],
        )
    }

    fn installable_rust_configuration() -> LanguageConfiguration {
        LanguageConfiguration::from_grammar(
            "rust",
            ["rs"],
            GrammarSource::new(
                "https://github.com/tree-sitter/tree-sitter-rust.git",
                ".",
                "src",
                "tree-sitter-rust",
                "tree_sitter_rust",
            ),
            [CaptureThemeMapping::new("keyword", "syntax.keyword")],
        )
    }

    fn rust_inline_configuration() -> LanguageConfiguration {
        LanguageConfiguration::new(
            "rust-inline",
            [] as [&str; 0],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [CaptureThemeMapping::new("string", "syntax.string.inline")],
        )
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    #[test]
    fn registry_resolves_languages_by_extension_and_path() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        assert_eq!(
            registry
                .language_for_extension(".rs")
                .map(|language| language.id()),
            Some("rust")
        );
        assert_eq!(
            registry
                .language_for_path("src\\main.rs")
                .map(|language| language.id()),
            Some("rust")
        );
    }

    #[test]
    fn rust_highlighting_produces_theme_tokens() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let buffer = TextBuffer::from_text(
            r#"
fn main() {
    let value = "volt";
    println!("{value}");
}
"#,
        );

        let snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
        assert_eq!(snapshot.language_id, "rust");
        assert_eq!(snapshot.root_kind, "source_file");
        assert!(!snapshot.has_errors);
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.keyword")
        );
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.string")
        );
    }

    #[test]
    fn ancestor_contexts_include_named_nodes_up_to_the_root() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let buffer = TextBuffer::from_text(
            r#"impl Demo {
    fn render(value: usize) {
        let current = value;
    }
}
"#,
        );
        let contexts =
            must(registry.ancestor_contexts_for_language("rust", &buffer, TextPoint::new(2, 8)));

        let kinds = contexts
            .iter()
            .map(|context| context.kind.as_str())
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"function_item"));
        assert!(kinds.contains(&"impl_item"));
        assert!(!kinds.contains(&"source_file"));
    }

    #[test]
    fn additional_highlight_languages_merge_spans() {
        let mut registry = SyntaxRegistry::new();
        must(
            registry.register(
                rust_configuration().with_additional_highlight_languages(["rust-inline"]),
            ),
        );
        must(registry.register(rust_inline_configuration()));

        let buffer = TextBuffer::from_text("fn main() { let value = \"volt\"; }");
        let snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.string.inline")
        );
    }

    #[test]
    fn visible_spans_filters_to_line_window() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut source = String::new();
        for _ in 0..256 {
            source.push_str("fn demo() {}\n");
        }
        source.push_str("let target = \"visible\";\n");
        for _ in 0..256 {
            source.push_str("fn tail() {}\n");
        }

        let snapshot =
            must(registry.highlight_buffer_for_extension("rs", &TextBuffer::from_text(source)));
        let visible = snapshot.visible_spans(256, 4);

        assert!(!visible.is_empty());
        assert!(visible.iter().all(|span| span.start_position.line <= 259));
    }

    #[test]
    fn highlight_window_limits_highlight_spans_to_requested_lines() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut source = String::new();
        for index in 0..512 {
            source.push_str(&format!(
                "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
            ));
        }
        let buffer = TextBuffer::from_text(source);

        let full_snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
        let window = HighlightWindow::new(240, 16);
        let windowed_snapshot =
            must(registry.highlight_buffer_for_extension_window("rs", &buffer, window));

        assert!(!windowed_snapshot.highlight_spans.is_empty());
        assert!(windowed_snapshot.highlight_spans.len() < full_snapshot.highlight_spans.len());
        assert!(windowed_snapshot.highlight_spans.iter().all(|span| {
            span.start_position.line < window.end_line_exclusive()
                && span.end_position.line >= window.start_line()
        }));
    }

    #[test]
    fn incremental_parse_session_matches_cold_highlight_after_edits() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut buffer = TextBuffer::from_text("fn main() {\n    let value = 1;\n}\n");
        let mut parse_session: Option<SyntaxParseSession> = None;
        let initial_snapshot = must(registry.highlight_buffer_for_language_with_session(
            "rust",
            &buffer,
            &mut parse_session,
        ));

        buffer.set_cursor(editor_buffer::TextPoint::new(1, 16));
        buffer.insert_text("mut ");
        let incremental_snapshot = must(registry.highlight_buffer_for_language_with_session(
            "rust",
            &buffer,
            &mut parse_session,
        ));
        let cold_snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));

        assert_eq!(incremental_snapshot, cold_snapshot);
        assert_ne!(initial_snapshot, incremental_snapshot);
    }

    #[test]
    fn incremental_windowed_session_matches_cold_highlight_after_edits() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut source = String::new();
        for index in 0..512 {
            source.push_str(&format!(
                "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
            ));
        }
        let mut buffer = TextBuffer::from_text(source);
        let window = HighlightWindow::new(240, 24);
        let mut parse_session: Option<SyntaxParseSession> = None;

        let _ = must(registry.highlight_buffer_for_language_window_with_session(
            "rust",
            &buffer,
            window,
            &mut parse_session,
        ));

        buffer.set_cursor(editor_buffer::TextPoint::new(248, 0));
        buffer.insert_text("x");

        let incremental_snapshot =
            must(registry.highlight_buffer_for_language_window_with_session(
                "rust",
                &buffer,
                window,
                &mut parse_session,
            ));
        let cold_snapshot =
            must(registry.highlight_buffer_for_language_window("rust", &buffer, window));

        assert_eq!(incremental_snapshot, cold_snapshot);
    }

    #[test]
    fn grammar_configuration_uses_install_root_paths() {
        let grammar = GrammarSource::new(
            "https://example.com/tree-sitter-rust.git",
            ".",
            "src",
            "tree-sitter-rust",
            "tree_sitter_rust",
        );
        let install_root = PathBuf::from("volt-grammars");

        assert_eq!(
            grammar.install_directory(&install_root),
            install_root.join("tree-sitter-rust")
        );
        assert_eq!(
            grammar.installed_highlight_query_path(&install_root),
            install_root
                .join("tree-sitter-rust")
                .join("queries")
                .join("highlights.scm")
        );
        assert!(
            grammar
                .installed_library_path(&install_root)
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(|file_name| file_name.starts_with("libtree-sitter-rust"))
                .unwrap_or(false)
        );
    }

    #[test]
    fn grammar_backed_language_reports_missing_install() {
        let install_root = std::env::temp_dir().join("volt-missing-tree-sitter-grammar");
        let mut registry = SyntaxRegistry::with_install_root(install_root.clone());
        must(registry.register(installable_rust_configuration()));

        assert!(!must(registry.is_installed("rust")));
        let error = registry
            .highlight_buffer_for_extension("rs", &TextBuffer::from_text("fn main() {}"))
            .expect_err("expected missing grammar error");
        match error {
            SyntaxError::GrammarNotInstalled {
                language_id,
                install_dir,
            } => {
                assert_eq!(language_id, "rust");
                assert_eq!(install_dir, install_root.join("tree-sitter-rust"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn extra_highlight_query_can_seed_missing_installed_query_file() {
        let query_path = std::env::temp_dir().join(format!(
            "volt-extra-query-{}-{}.scm",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
        ));
        let config =
            installable_rust_configuration().with_extra_highlight_query("(identifier) @function");

        must(ensure_installed_highlight_query_path(&config, &query_path));

        assert!(query_path.exists());
        assert_eq!(std::fs::read_to_string(&query_path).unwrap_or_default(), "");

        if query_path.exists() {
            let _ = std::fs::remove_file(&query_path);
        }
    }
}
