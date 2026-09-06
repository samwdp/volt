use crate::install::*;
use crate::query::*;
use crate::registry::*;

use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use editor_buffer::{SyntaxText, TextByteChunks};
use editor_path::PathMatcher;
pub use tree_sitter::Language;
use tree_sitter::{Parser, TextProvider, Tree};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Tree-sitter language registration, installation, parsing, highlighting, and indentation.";

#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) const DEFAULT_QUERY_ASSET_SEARCH_DEPTH: usize = 6;

pub(crate) const MAX_INJECTION_DEPTH: usize = 8;

pub(crate) const INDENT_QUERY_MATCH_LIMIT: u32 = 1024;

pub(crate) const INDENT_QUERY_PROGRESS_LIMIT: usize = 100_000;

pub(crate) const QUERY_ASSET_DIR_CANDIDATES: &[&[&str]] = &[
    &["crates", "volt", "assets", "grammars", "queries"],
    &["assets", "grammars", "queries"],
];

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

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

/// One external command needed to install a grammar-backed language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallCommandSpec {
    pub(crate) label: String,
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) env: Vec<(String, String)>,
}

/// One failed grammar recompile entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarRecompileFailure {
    pub(crate) language_id: String,
    pub(crate) message: String,
}

impl GrammarRecompileFailure {
    /// Returns the language id that failed to recompile.
    pub fn language_id(&self) -> &str {
        &self.language_id
    }

    /// Returns the failure message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Result of a best-effort installed grammar recompile pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GrammarRecompileReport {
    pub(crate) recompiled: Vec<String>,
    pub(crate) failed: Vec<GrammarRecompileFailure>,
}

impl GrammarRecompileReport {
    /// Returns language ids that recompiled successfully.
    pub fn recompiled(&self) -> &[String] {
        &self.recompiled
    }

    /// Returns failed language ids with error messages.
    pub fn failed(&self) -> &[GrammarRecompileFailure] {
        &self.failed
    }

    /// Reports whether every attempted grammar recompiled successfully.
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

impl InstallCommandSpec {
    pub(crate) fn new(
        label: impl Into<String>,
        program: impl Into<String>,
        args: Vec<String>,
        cwd: impl Into<PathBuf>,
    ) -> Self {
        Self {
            label: label.into(),
            program: program.into(),
            args,
            cwd: cwd.into(),
            env: Vec::new(),
        }
    }

    #[cfg(windows)]
    pub(crate) fn with_env<I, K, V>(mut self, env: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.env = env
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        self
    }

    /// Returns the human-readable command label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the executable name.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Returns the command-line arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns the command working directory.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Returns environment variables required by the command.
    pub fn env(&self) -> &[(String, String)] {
        &self.env
    }
}

/// Reusable install metadata for one grammar-backed language installation.
#[derive(Debug)]
pub struct LanguageInstallPlan {
    pub(crate) config: LanguageConfiguration,
    pub(crate) grammar: GrammarSource,
    pub(crate) install_root: PathBuf,
    pub(crate) query_asset_root: Option<PathBuf>,
    pub(crate) temp_clone_root: PathBuf,
}

impl LanguageInstallPlan {
    /// Returns the registered language id being installed.
    pub fn language_id(&self) -> &str {
        self.config.id()
    }

    /// Returns the temporary clone root used for this install.
    pub fn clone_root(&self) -> &Path {
        &self.temp_clone_root
    }

    /// Returns the cloned grammar directory inside the temporary checkout.
    pub fn grammar_dir(&self) -> PathBuf {
        self.temp_clone_root.join(self.grammar.grammar_dir())
    }

    /// Returns the source directory used by the generated parser and optional scanners.
    pub fn source_dir(&self) -> PathBuf {
        self.grammar_dir().join(self.grammar.source_dir())
    }

    /// Returns the grammar library install root.
    pub fn install_dir(&self) -> PathBuf {
        self.install_root.clone()
    }

    /// Returns the installed shared library path for this grammar.
    pub fn installed_library_path(&self) -> PathBuf {
        self.grammar.installed_library_path(&self.install_root)
    }

    /// Ensures the temporary clone parent exists before launching `git clone`.
    pub fn prepare_clone_root(&self) -> Result<(), SyntaxError> {
        let parent = self
            .temp_clone_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(std::env::temp_dir);
        fs::create_dir_all(&parent).map_err(|error| io_error("create temp parent", &parent, error))
    }

    /// Returns the `git clone` command needed to populate the temporary checkout.
    pub fn clone_command(&self) -> InstallCommandSpec {
        InstallCommandSpec::new(
            format!(
                "git clone --depth 1 {} {}",
                self.grammar.repository_url(),
                self.temp_clone_root.display()
            ),
            "git",
            vec![
                "clone".to_owned(),
                "--depth".to_owned(),
                "1".to_owned(),
                self.grammar.repository_url().to_owned(),
                self.temp_clone_root.display().to_string(),
            ],
            self.temp_clone_root
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(std::env::temp_dir),
        )
    }

    /// Creates the install directory and verifies bundled query assets are available.
    pub fn prepare_install_root(&self) -> Result<(), SyntaxError> {
        ensure_cloned_grammar_dir_exists(&self.grammar_dir())?;
        fs::create_dir_all(&self.install_root)
            .map_err(|error| io_error("create grammar install root", &self.install_root, error))?;
        remove_legacy_grammar_install_directory(&self.grammar, &self.install_root)?;
        let query_asset_root = self
            .query_asset_root
            .as_deref()
            .ok_or_else(|| SyntaxError::Io {
                operation: "resolve bundled query asset root".to_owned(),
                path: self.install_root.clone(),
                message: "bundled tree-sitter query assets are not configured".to_owned(),
            })?;
        if self.config.extra_highlight_query().is_none()
            && resolve_bundled_query_source(
                query_asset_root,
                self.config.id(),
                "highlights.scm",
                &mut Vec::new(),
            )?
            .is_none()
        {
            return Err(SyntaxError::Io {
                operation: "locate bundled highlight query".to_owned(),
                path: query_asset_root
                    .join(self.config.id())
                    .join("highlights.scm"),
                message: "bundled highlights.scm is missing for this language".to_owned(),
            });
        }
        Ok(())
    }

    /// Returns the `tree-sitter generate` command when the cloned grammar still needs `parser.c`.
    pub fn generate_command_if_needed(&self) -> Result<Option<InstallCommandSpec>, SyntaxError> {
        ensure_cloned_grammar_dir_exists(&self.grammar_dir())?;
        let parser_path = self.source_dir().join("parser.c");
        if parser_path.exists() {
            return Ok(None);
        }
        let grammar_js_path = self.grammar_dir().join("grammar.js");
        if !grammar_js_path.exists() {
            return Err(SyntaxError::Io {
                operation: "locate grammar source".to_owned(),
                path: grammar_js_path,
                message: "grammar.js is missing and parser.c was not pre-generated".to_owned(),
            });
        }
        Ok(Some(InstallCommandSpec::new(
            "tree-sitter generate",
            "tree-sitter",
            vec!["generate".to_owned()],
            self.grammar_dir(),
        )))
    }

    /// Returns the compiler command for the cloned grammar after `parser.c` is available.
    pub fn compile_command(&self) -> Result<InstallCommandSpec, SyntaxError> {
        ensure_cloned_grammar_dir_exists(&self.grammar_dir())?;
        let source_dir = self.source_dir();
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
        let output_path = self.grammar.installed_library_path(&self.install_root);
        #[cfg(windows)]
        {
            let target = windows_msvc_target_triple();
            let compiler =
                find_msvc_tools::find_tool(target, "cl.exe").ok_or_else(|| SyntaxError::Io {
                    operation: "locate MSVC compiler".to_owned(),
                    path: self.install_root.clone(),
                    message: format!("could not find cl.exe for {target}"),
                })?;
            let mut env = compiler
                .env()
                .into_iter()
                .map(|(key, value)| {
                    (
                        key.to_string_lossy().into_owned(),
                        value.to_string_lossy().into_owned(),
                    )
                })
                .collect::<Vec<_>>();
            env.sort_by(|left, right| left.0.cmp(&right.0));
            let program = compiler.path().display().to_string();
            let mut args = vec![
                "/nologo".to_owned(),
                "/LD".to_owned(),
                "/I".to_owned(),
                source_dir.display().to_string(),
                format!("/Fo{}{}", source_dir.display(), std::path::MAIN_SEPARATOR),
            ];
            if scanner_cpp.exists() {
                args.push("/EHsc".to_owned());
            }
            args.push(parser_path.display().to_string());
            if scanner_c.exists() {
                args.push(scanner_c.display().to_string());
            }
            if scanner_cpp.exists() {
                args.push(scanner_cpp.display().to_string());
            }
            args.push(format!("/Fe:{}", output_path.display()));
            args.extend(["/link".to_owned(), "/NOIMPLIB".to_owned()]);
            Ok(InstallCommandSpec::new(
                format!("{} {}", program, args.join(" ")),
                program,
                args,
                self.grammar_dir(),
            )
            .with_env(env))
        }
        #[cfg(not(windows))]
        {
            let compiler = if scanner_cpp.exists() { "c++" } else { "cc" };
            let mut args = Vec::new();
            if cfg!(target_os = "macos") {
                args.extend(["-fPIC".to_owned(), "-dynamiclib".to_owned()]);
            } else {
                args.extend(["-fPIC".to_owned(), "-shared".to_owned()]);
            }
            if scanner_cpp.exists() {
                args.push("-std=c++14".to_owned());
            }
            args.push(parser_path.display().to_string());
            if scanner_c.exists() {
                args.push(scanner_c.display().to_string());
            }
            if scanner_cpp.exists() {
                args.push(scanner_cpp.display().to_string());
            }
            args.push("-I".to_owned());
            args.push(source_dir.display().to_string());
            args.push("-o".to_owned());
            args.push(output_path.display().to_string());
            Ok(InstallCommandSpec::new(
                format!("{compiler} {}", args.join(" ")),
                compiler,
                args,
                self.grammar_dir(),
            ))
        }
    }
}

impl Drop for LanguageInstallPlan {
    fn drop(&mut self) {
        if self.temp_clone_root.exists() {
            let _ = fs::remove_dir_all(&self.temp_clone_root);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum LanguageLoader {
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

    pub(crate) fn path_match_score(&self, path: &Path) -> Option<usize> {
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
    pub capture_name: Arc<str>,
    /// Resolved theme token.
    pub theme_token: Arc<str>,
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

/// A named tree-sitter node with byte/position span for structure walks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxStructureNode {
    /// Tree-sitter node kind.
    pub kind: String,
    /// Start byte offset.
    pub start_byte: usize,
    /// End byte offset.
    pub end_byte: usize,
    /// Starting line/column for the node.
    pub start_position: SyntaxPoint,
    /// Exclusive ending line/column for the node.
    pub end_position: SyntaxPoint,
}

/// Reusable parser/tree state for incremental highlighting of one buffer.
pub struct SyntaxParseSession {
    pub(crate) language_id: String,
    pub(crate) revision: u64,
    pub(crate) parser: Parser,
    pub(crate) tree: Tree,
    pub(crate) last_highlight_window: Option<HighlightWindow>,
    pub(crate) last_snapshot: Option<SyntaxSnapshot>,
}

/// A requested line window for range-limited syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightWindow {
    pub(crate) start_line: usize,
    pub(crate) line_count: usize,
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

    pub(crate) const fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    pub(crate) const fn end_line_exclusive(&self) -> usize {
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
    /// A query failed to compile.
    InvalidQuery {
        language_id: String,
        query_kind: String,
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
                query_kind,
                message,
            } => {
                write!(
                    formatter,
                    "{query_kind} query for `{language_id}` is invalid: {message}"
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

impl<'a, B: SyntaxText + ?Sized> TextProvider<&'a [u8]> for SyntaxTextProvider<'a, B> {
    type I = TextByteChunks<'a>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        self.buffer.byte_slice_chunks(node.byte_range())
    }
}
