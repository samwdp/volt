#![doc = r#"Tree-sitter language registration, installation, parsing, and capture-to-theme mapping."#]

use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use editor_buffer::TextBuffer;
pub use tree_sitter::Language;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};
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

    /// Returns the stable install directory name used under `user/lang/grammars`.
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
    /// Creates a syntax registry using the default `user/lang/grammars` install root.
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
        copy_dir_all(&cloned_grammar_dir, &install_dir)?;
        build_shared_library(language_id, &grammar, &install_dir)?;

        self.loaded.remove(language_id);
        Ok(install_dir)
    }

    /// Parses and highlights a buffer for a known file extension.
    pub fn highlight_buffer_for_extension(
        &mut self,
        extension: &str,
        buffer: &TextBuffer,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        let extension = normalize_extension(extension);
        let language_id = self
            .extensions
            .get(&extension)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownExtension(extension.clone()))?;
        self.ensure_loaded_language(&language_id)?;
        let loaded = self
            .loaded
            .get(&language_id)
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        highlight_loaded_language(&language_id, loaded, buffer)
    }

    /// Parses and highlights a buffer using a file path's extension.
    pub fn highlight_buffer_for_path(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &TextBuffer,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| SyntaxError::UnknownExtension(path.display().to_string()))?;
        self.highlight_buffer_for_extension(extension, buffer)
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
            let query = Query::new(&language, highlight_query).map_err(|error| {
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

            let query_source = fs::read_to_string(&query_path)
                .map_err(|error| io_error("read highlight query", &query_path, error))?;
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

fn highlight_loaded_language(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
) -> Result<SyntaxSnapshot, SyntaxError> {
    let mut parser = Parser::new();
    parser
        .set_language(&loaded.language)
        .map_err(|error| SyntaxError::ParserConfiguration {
            language_id: language_id.to_owned(),
            message: error.to_string(),
        })?;

    let source = buffer.text();
    let tree = parser
        .parse(&source, None)
        .ok_or_else(|| SyntaxError::ParseCancelled(language_id.to_owned()))?;

    let mut query_cursor = QueryCursor::new();
    let capture_names = loaded.query.capture_names();
    let mut highlight_spans = Vec::new();

    let mut matches = query_cursor.matches(&loaded.query, tree.root_node(), source.as_bytes());
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

    Ok(SyntaxSnapshot {
        language_id: language_id.to_owned(),
        root_kind: tree.root_node().kind().to_owned(),
        has_errors: tree.root_node().has_error(),
        highlight_spans,
    })
}

fn build_shared_library(
    language_id: &str,
    grammar: &GrammarSource,
    install_dir: &Path,
) -> Result<(), SyntaxError> {
    let source_dir = install_dir.join(grammar.source_dir());
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
    let output_path =
        grammar.installed_library_path(install_dir.parent().unwrap_or_else(|| Path::new(".")));
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
    command.current_dir(install_dir);

    let output = command
        .output()
        .map_err(|error| io_error("run grammar compiler", install_dir, error))?;
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
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("user")
        .join("lang")
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
    use std::path::PathBuf;

    use super::{
        CaptureThemeMapping, GrammarSource, LanguageConfiguration, SyntaxError, SyntaxRegistry,
    };
    use editor_buffer::TextBuffer;

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
    fn grammar_configuration_uses_install_root_paths() {
        let grammar = GrammarSource::new(
            "https://example.com/tree-sitter-rust.git",
            ".",
            "src",
            "tree-sitter-rust",
            "tree_sitter_rust",
        );
        let install_root = PathBuf::from("P:\\volt\\user\\lang\\grammars");

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
}
