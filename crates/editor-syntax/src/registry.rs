use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    mem::ManuallyDrop,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use editor_buffer::{SyntaxText, TextBuffer, TextEdit, TextPoint};
use tree_sitter::Language;
use tree_sitter::{Query, Range, Tree};

use crate::highlight::*;
use crate::install::*;
use crate::language::*;
use crate::query::*;

pub(crate) struct LoadedLanguage {
    pub(crate) query: Query,
    pub(crate) indent_query: DeferredQuery,
    pub(crate) injections_query: DeferredQuery,
    pub(crate) locals_query: DeferredQuery,
    pub(crate) folds_query: DeferredQuery,
    pub(crate) capture_names: Vec<Arc<str>>,
    pub(crate) capture_tokens: Vec<Arc<str>>,
    // Query values must drop before their language handle.
    pub(crate) language: Language,
    // Must drop after every tree-sitter value above; destructors may still access grammar data.
    pub(crate) _library: Option<ManuallyDrop<libloading::Library>>,
}

pub(crate) struct DeferredQuery {
    pub(crate) kind_label: &'static str,
    pub(crate) source: Option<String>,
    pub(crate) compiled: OnceLock<Option<Query>>,
}

pub(crate) struct ParsedHighlight {
    pub(crate) snapshot: SyntaxSnapshot,
    /// Owned tree when no parse session holds it. Session trees stay in the session.
    pub(crate) tree: Option<Tree>,
}

impl ParsedHighlight {
    pub(crate) fn tree<'a>(&'a self, session: Option<&'a SyntaxParseSession>) -> Option<&'a Tree> {
        self.tree
            .as_ref()
            .or_else(|| session.map(|session| &session.tree))
    }
}

pub(crate) struct ParseTreeResult {
    /// Owned tree when highlighting without a parse session.
    pub(crate) owned_tree: Option<Tree>,
    pub(crate) changed_ranges: Option<Vec<Range>>,
    pub(crate) applied_edits: Option<Vec<TextEdit>>,
}

impl ParseTreeResult {
    pub(crate) fn tree<'a>(&'a self, session: Option<&'a SyntaxParseSession>) -> Option<&'a Tree> {
        self.owned_tree
            .as_ref()
            .or_else(|| session.map(|session| &session.tree))
    }
}

#[derive(Default)]
pub(crate) struct InjectionHighlights {
    pub(crate) highlight_spans: Vec<HighlightSpan>,
    pub(crate) has_errors: bool,
}

pub(crate) struct InjectionRegion {
    pub(crate) language_name: String,
    pub(crate) start_byte: usize,
    pub(crate) end_byte: usize,
    pub(crate) start_position: SyntaxPoint,
    pub(crate) end_position: SyntaxPoint,
}

pub(crate) struct SyntaxTextProvider<'a, B: SyntaxText + ?Sized> {
    pub(crate) buffer: &'a B,
}

impl LoadedLanguage {
    pub(crate) fn interned_capture(&self, index: usize) -> Option<(&Arc<str>, &Arc<str>)> {
        Some((
            self.capture_names.get(index)?,
            self.capture_tokens.get(index)?,
        ))
    }

    pub(crate) fn has_injections(&self) -> bool {
        self.injections_query.has_source()
    }
}

impl DeferredQuery {
    pub(crate) fn from_source(kind_label: &'static str, source: Option<String>) -> Self {
        Self {
            kind_label,
            source,
            compiled: OnceLock::new(),
        }
    }

    pub(crate) fn has_source(&self) -> bool {
        self.source.is_some() || self.compiled.get().is_some_and(Option::is_some)
    }

    pub(crate) fn query<'a>(
        &'a self,
        language: &Language,
        language_id: &str,
    ) -> Result<Option<&'a Query>, SyntaxError> {
        if self.compiled.get().is_none() {
            let compiled = match self.source.as_deref() {
                Some(source) => Some(compile_query_source(
                    language,
                    language_id,
                    self.kind_label,
                    source,
                )?),
                None => None,
            };
            let _ = self.compiled.set(compiled);
        }
        Ok(self.compiled.get().and_then(|query| query.as_ref()))
    }
}

pub(crate) fn capture_requires_theme_token(capture_name: &str) -> bool {
    !capture_name.starts_with('_')
        && !matches!(
            capture_name,
            "spell" | "nospell" | "conceal" | "conceal_lines"
        )
}

/// Runtime registry of known tree-sitter languages.
pub struct SyntaxRegistry {
    pub(crate) install_root: PathBuf,
    pub(crate) query_asset_root: Option<PathBuf>,
    pub(crate) languages: BTreeMap<String, LanguageConfiguration>,
    pub(crate) language_order: Vec<String>,
    pub(crate) extensions: BTreeMap<String, String>,
    pub(crate) loaded: BTreeMap<String, LoadedLanguage>,
}

impl fmt::Debug for SyntaxRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SyntaxRegistry")
            .field("install_root", &self.install_root)
            .field("query_asset_root", &self.query_asset_root)
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
            query_asset_root: default_query_asset_root(),
            languages: BTreeMap::new(),
            language_order: Vec::new(),
            extensions: BTreeMap::new(),
            loaded: BTreeMap::new(),
        }
    }

    /// Returns the grammar install root.
    pub fn install_root(&self) -> &Path {
        &self.install_root
    }

    /// Returns the bundled query asset root, when configured.
    pub fn query_asset_root(&self) -> Option<&Path> {
        self.query_asset_root.as_deref()
    }

    /// Replaces the bundled query asset root used for grammar query installation/loading.
    pub fn set_query_asset_root(&mut self, query_asset_root: Option<PathBuf>) {
        self.query_asset_root = query_asset_root;
        self.loaded.clear();
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
        self.language_order.push(language_id.clone());
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
        let mut best = None;
        let mut best_score = 0;
        for language_id in &self.language_order {
            let Some(language) = self.languages.get(language_id) else {
                continue;
            };
            let Some(score) = language.path_match_score(path) else {
                continue;
            };
            if best.is_none() || score > best_score {
                best = Some(language);
                best_score = score;
            }
        }
        best
    }

    /// Reports whether a grammar-backed language is installed.
    pub fn is_installed(&self, language_id: &str) -> Result<bool, SyntaxError> {
        let Some(config) = self.languages.get(language_id) else {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        };

        Ok(match config.grammar() {
            Some(grammar) => grammar.installed_library_path(&self.install_root).exists(),
            None => true,
        })
    }

    /// Returns installed grammar-backed language ids in registration order.
    pub fn installed_grammar_language_ids(&self) -> Vec<String> {
        let mut seen_libraries = BTreeSet::new();
        self.language_order
            .iter()
            .filter_map(|language_id| {
                let config = self.languages.get(language_id)?;
                let grammar = config.grammar()?;
                let library_path = grammar.installed_library_path(&self.install_root);
                (library_path.exists() && seen_libraries.insert(library_path))
                    .then(|| language_id.clone())
            })
            .collect()
    }

    /// Reports whether a language's grammar and highlight query are already loaded.
    pub fn is_loaded(&self, language_id: &str) -> bool {
        self.loaded.contains_key(language_id)
    }

    /// Loads a language's grammar and highlight query without parsing a buffer.
    pub fn preload_language(&mut self, language_id: &str) -> Result<(), SyntaxError> {
        self.ensure_loaded_language(language_id)
    }

    /// Builds a reusable install plan for one grammar-backed language.
    pub fn prepare_language_install(
        &self,
        language_id: &str,
    ) -> Result<Option<LanguageInstallPlan>, SyntaxError> {
        let config = self
            .languages
            .get(language_id)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.to_owned()))?;
        let Some(grammar) = config.grammar().cloned() else {
            return Ok(None);
        };

        let temp_clone_root = std::env::temp_dir().join(format!(
            "volt-treesitter-{}",
            temp_guid_like_directory_name()
        ));
        Ok(Some(LanguageInstallPlan {
            config,
            grammar,
            install_root: self.install_root.clone(),
            query_asset_root: self.query_asset_root.clone(),
            temp_clone_root,
        }))
    }

    /// Drops any cached loaded grammar/query state for the given language.
    pub fn invalidate_language(&mut self, language_id: &str) -> Result<(), SyntaxError> {
        if !self.languages.contains_key(language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        }
        self.loaded.remove(language_id);
        Ok(())
    }

    /// Installs a grammar-backed language into the configured install root.
    pub fn install_language(&mut self, language_id: &str) -> Result<PathBuf, SyntaxError> {
        let Some(install_plan) = self.prepare_language_install(language_id)? else {
            return Ok(self.install_root.clone());
        };
        install_plan.prepare_clone_root()?;
        run_install_command(language_id, &install_plan.clone_command())?;
        install_plan.prepare_install_root()?;
        if let Some(generate_command) = install_plan.generate_command_if_needed()? {
            run_install_command(language_id, &generate_command)?;
        }
        run_install_command(language_id, &install_plan.compile_command()?)?;
        remove_compiler_sidecar_artifacts(
            &install_plan
                .grammar
                .installed_library_path(&install_plan.install_root),
        )?;
        let install_dir = install_plan.install_dir();
        self.invalidate_language(language_id)?;
        Ok(install_dir)
    }

    /// Finalizes one grammar-backed language install after external commands have run.
    pub fn finalize_language_install(
        &mut self,
        install_plan: &LanguageInstallPlan,
    ) -> Result<(), SyntaxError> {
        remove_compiler_sidecar_artifacts(&install_plan.installed_library_path())?;
        self.invalidate_language(install_plan.language_id())
    }

    /// Recompiles all currently installed grammar-backed languages.
    pub fn recompile_installed_languages(&mut self) -> Result<Vec<String>, SyntaxError> {
        let language_ids = self.installed_grammar_language_ids();
        for language_id in &language_ids {
            self.install_language(language_id)?;
        }
        Ok(language_ids)
    }

    /// Recompiles all currently installed grammar-backed languages, continuing after failures.
    pub fn recompile_installed_languages_best_effort(&mut self) -> GrammarRecompileReport {
        let mut report = GrammarRecompileReport::default();
        for language_id in self.installed_grammar_language_ids() {
            match self.install_language(&language_id) {
                Ok(_) => report.recompiled.push(language_id),
                Err(error) => report.failed.push(GrammarRecompileFailure {
                    language_id,
                    message: error.to_string(),
                }),
            }
        }
        report
    }

    /// Parses and highlights a buffer for a known file extension.
    pub fn highlight_buffer_for_extension(
        &mut self,
        extension: &str,
        buffer: &impl SyntaxText,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_extension_impl(extension, buffer, None, None)
    }

    /// Parses and highlights a line window for a known file extension.
    pub fn highlight_buffer_for_extension_window(
        &mut self,
        extension: &str,
        buffer: &impl SyntaxText,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_extension_impl(extension, buffer, Some(highlight_window), None)
    }

    pub(crate) fn highlight_buffer_for_extension_impl(
        &mut self,
        extension: &str,
        buffer: &impl SyntaxText,
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
        buffer: &impl SyntaxText,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, None, None)
    }

    /// Walks the parse tree and returns every named node (pre-order).
    pub fn structure_nodes_for_language(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
    ) -> Result<Vec<SyntaxStructureNode>, SyntaxError> {
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
        let tree = require_tree(&language_id, &parse_result, None)?;
        Ok(collect_structure_nodes(tree.root_node()))
    }

    /// Returns named ancestor nodes for a cursor location, ordered innermost to outermost.
    pub fn ancestor_contexts_for_language(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        point: TextPoint,
    ) -> Result<Vec<SyntaxNodeContext>, SyntaxError> {
        self.ancestor_contexts_for_language_impl(language_id, buffer, point, None)
    }

    /// Returns named ancestor nodes for a cursor location, ordered innermost to outermost,
    /// reusing an existing parse session when provided.
    pub fn ancestor_contexts_for_language_with_parse_session(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        point: TextPoint,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<Vec<SyntaxNodeContext>, SyntaxError> {
        self.ancestor_contexts_for_language_impl(language_id, buffer, point, Some(parse_session))
    }

    pub(crate) fn ancestor_contexts_for_language_impl(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        point: TextPoint,
        mut parse_session: Option<&mut Option<SyntaxParseSession>>,
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
        let parse_result = parse_tree(&language_id, loaded, buffer, parse_session.as_deref_mut())?;
        let tree = require_tree(
            &language_id,
            &parse_result,
            parse_session_ref(parse_session.as_deref()),
        )?;
        let point = text_point_to_tree_sitter_point(buffer, point);
        let Some(mut node) = tree
            .root_node()
            .named_descendant_for_point_range(point, point)
        else {
            return Ok(Vec::new());
        };
        let mut contexts = Vec::new();
        loop {
            let start = node.start_position();
            let end = node.end_position();
            let parent = node.parent();
            // Only keep ancestors whose closing line is at or after the cursor so
            // callers can render closing-line context breadcrumbs.
            if node.is_named() && parent.is_some() && end.row >= point.row {
                contexts.push(SyntaxNodeContext {
                    kind: node.kind().to_owned(),
                    start_position: SyntaxPoint::new(start.row, start.column),
                    end_position: SyntaxPoint::new(end.row, end.column),
                });
            }
            let Some(parent) = parent else {
                break;
            };
            node = parent;
        }
        Ok(contexts)
    }

    /// Returns the desired indentation column for a target line when an indent query is available.
    pub fn desired_indent_for_language(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        line_index: usize,
        indent_width: usize,
    ) -> Result<Option<usize>, SyntaxError> {
        self.desired_indent_for_language_impl(language_id, buffer, line_index, indent_width, None)
    }

    /// Returns the desired indentation column using a reusable parse session.
    pub fn desired_indent_for_language_with_session(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        line_index: usize,
        indent_width: usize,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<Option<usize>, SyntaxError> {
        self.desired_indent_for_language_impl(
            language_id,
            buffer,
            line_index,
            indent_width,
            Some(parse_session),
        )
    }

    pub(crate) fn desired_indent_for_language_impl(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        line_index: usize,
        indent_width: usize,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<Option<usize>, SyntaxError> {
        let language_id = language_id.to_owned();
        if !self.languages.contains_key(&language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id));
        }
        self.ensure_loaded_language(&language_id)?;
        let loaded = self
            .loaded
            .get(&language_id)
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        desired_indent_for_loaded_language(
            &language_id,
            loaded,
            buffer,
            line_index,
            indent_width,
            parse_session,
        )
    }

    /// Parses and highlights a line window using a registered language identifier.
    pub fn highlight_buffer_for_language_window(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, Some(highlight_window), None)
    }

    /// Parses and highlights a buffer using a reusable parse session.
    pub fn highlight_buffer_for_language_with_session(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, None, Some(parse_session))
    }

    /// Parses and highlights a line window using a reusable parse session.
    pub fn highlight_buffer_for_language_window_with_session(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
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

    pub(crate) fn highlight_buffer_for_language_impl(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        highlight_window: Option<HighlightWindow>,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl_with_depth(
            language_id,
            buffer,
            highlight_window,
            parse_session,
            0,
        )
    }

    pub(crate) fn highlight_buffer_for_language_impl_with_depth(
        &mut self,
        language_id: &str,
        buffer: &impl SyntaxText,
        highlight_window: Option<HighlightWindow>,
        mut parse_session: Option<&mut Option<SyntaxParseSession>>,
        injection_depth: usize,
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
        let has_injections = self
            .loaded
            .get(&language_id)
            .map(LoadedLanguage::has_injections)
            .unwrap_or(false);

        let base_parse = if needs_inline_ranges || has_injections {
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
                parse_session.as_deref_mut(),
            )?
        };

        if let Some(parse) = base_parse.as_ref().filter(|_| has_injections) {
            let session = parse_session_ref(parse_session.as_deref());
            let Some(tree) = parse.tree(session) else {
                return Err(SyntaxError::ParseCancelled(language_id));
            };
            let injections = self.highlight_injections_for_tree(
                &config,
                &language_id,
                tree,
                buffer,
                highlight_window,
                injection_depth,
            )?;
            snapshot.highlight_spans.extend(injections.highlight_spans);
            snapshot.has_errors = snapshot.has_errors || injections.has_errors;
        }

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
                let session = parse_session_ref(parse_session.as_deref());
                let Some(tree) = parse.tree(session) else {
                    continue;
                };
                let mut inline_lines = markdown_inline_line_indices(tree);
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

        sort_highlight_spans(&mut snapshot.highlight_spans);
        Ok(snapshot)
    }

    pub(crate) fn highlight_injections_for_tree(
        &mut self,
        host_config: &LanguageConfiguration,
        host_language_id: &str,
        tree: &Tree,
        buffer: &impl SyntaxText,
        highlight_window: Option<HighlightWindow>,
        injection_depth: usize,
    ) -> Result<InjectionHighlights, SyntaxError> {
        if injection_depth >= MAX_INJECTION_DEPTH {
            return Ok(InjectionHighlights::default());
        }

        let injection_regions = {
            let Some(loaded) = self.loaded.get(host_language_id) else {
                return Ok(InjectionHighlights::default());
            };
            let Some(injections_query) = loaded
                .injections_query
                .query(&loaded.language, host_language_id)?
            else {
                return Ok(InjectionHighlights::default());
            };
            collect_injection_regions(injections_query, tree, buffer, highlight_window)
        };

        let mut highlights = InjectionHighlights::default();
        for region in injection_regions {
            if let Some(window) = highlight_window
                && !injection_region_intersects_window(&region, window)
            {
                continue;
            }

            let Some(injection_language_id) =
                self.resolve_injection_language_id(&region.language_name)
            else {
                continue;
            };
            // A self-injection covering the entire current buffer cannot make progress. This can
            // happen when a query's `#offset!` directive is unavailable to this query runner: the
            // injected parser sees the same tagged template again and recursively reinjects it.
            if injection_language_id == host_language_id
                && region.start_byte == 0
                && region.end_byte == buffer.byte_count()
            {
                continue;
            }
            if host_config
                .additional_highlight_languages()
                .iter()
                .any(|language| language == &injection_language_id)
            {
                continue;
            }

            let Some(source) =
                buffer_text_for_byte_range(buffer, region.start_byte, region.end_byte)
            else {
                continue;
            };
            let Ok(snapshot) = self.highlight_buffer_for_language_impl_with_depth(
                &injection_language_id,
                &TextBuffer::from_text(source),
                None,
                None,
                injection_depth + 1,
            ) else {
                continue;
            };

            highlights.has_errors = highlights.has_errors || snapshot.has_errors;
            highlights.highlight_spans.extend(
                snapshot
                    .highlight_spans
                    .into_iter()
                    .map(|span| translate_injected_highlight_span(span, &region))
                    .filter(|span| {
                        highlight_window
                            .map(|window| span_intersects_window(span, window))
                            .unwrap_or(true)
                    }),
            );
        }

        Ok(highlights)
    }

    pub(crate) fn resolve_injection_language_id(&self, raw_language: &str) -> Option<String> {
        let raw_language = raw_language.trim();
        if raw_language.is_empty() {
            return None;
        }

        let mut candidates = Vec::new();
        for candidate in [
            raw_language.to_owned(),
            raw_language.to_ascii_lowercase(),
            raw_language.replace('_', "-"),
            raw_language.to_ascii_lowercase().replace('_', "-"),
        ] {
            if !candidate.is_empty() && !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        }

        for candidate in &candidates {
            if self.languages.contains_key(candidate) {
                return Some(candidate.clone());
            }
        }
        for candidate in &candidates {
            if let Some(language) = self.language_for_extension(candidate) {
                return Some(language.id().to_owned());
            }
        }
        for candidate in &candidates {
            if let Some(language) = self.language_for_path(candidate) {
                return Some(language.id().to_owned());
            }
        }
        None
    }

    /// Returns the compiled injections query for a language, loading it if needed.
    ///
    /// Returns `Ok(None)` when the language is registered but has no injections query.
    pub fn injections_query_for_language(
        &mut self,
        language_id: &str,
    ) -> Result<Option<&Query>, SyntaxError> {
        if !self.languages.contains_key(language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        }
        self.ensure_loaded_language(language_id)?;
        let Some(loaded) = self.loaded.get(language_id) else {
            return Ok(None);
        };
        loaded.injections_query.query(&loaded.language, language_id)
    }

    /// Returns the compiled locals query for a language, loading it if needed.
    ///
    /// Returns `Ok(None)` when the language is registered but has no locals query.
    pub fn locals_query_for_language(
        &mut self,
        language_id: &str,
    ) -> Result<Option<&Query>, SyntaxError> {
        if !self.languages.contains_key(language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        }
        self.ensure_loaded_language(language_id)?;
        let Some(loaded) = self.loaded.get(language_id) else {
            return Ok(None);
        };
        loaded.locals_query.query(&loaded.language, language_id)
    }

    /// Returns the compiled folds query for a language, loading it if needed.
    ///
    /// Returns `Ok(None)` when the language is registered but has no folds query.
    pub fn folds_query_for_language(
        &mut self,
        language_id: &str,
    ) -> Result<Option<&Query>, SyntaxError> {
        if !self.languages.contains_key(language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        }
        self.ensure_loaded_language(language_id)?;
        let Some(loaded) = self.loaded.get(language_id) else {
            return Ok(None);
        };
        loaded.folds_query.query(&loaded.language, language_id)
    }

    /// Parses and highlights a buffer using a file path's extension.
    pub fn highlight_buffer_for_path(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &impl SyntaxText,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_path_impl(path, buffer, None, None)
    }

    /// Parses and highlights a line window using a file path's extension.
    pub fn highlight_buffer_for_path_window(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &impl SyntaxText,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_path_impl(path, buffer, Some(highlight_window), None)
    }

    pub(crate) fn highlight_buffer_for_path_impl(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &impl SyntaxText,
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

    pub(crate) fn ensure_loaded_language(&mut self, language_id: &str) -> Result<(), SyntaxError> {
        if self.loaded.contains_key(language_id) {
            return Ok(());
        }

        let config = self
            .languages
            .get(language_id)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.to_owned()))?;
        let loaded = load_language(
            &config,
            &self.install_root,
            self.query_asset_root.as_deref(),
        )?;
        self.loaded.insert(language_id.to_owned(), loaded);
        Ok(())
    }
}
