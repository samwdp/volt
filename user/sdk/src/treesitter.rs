use std::cell::RefCell;

use tree_sitter::{Language, Parser};
use tree_sitter_language::LanguageFn;

use crate::path::grammar_install_root;
use crate::syntax::{LanguageConfiguration, LanguageLoader, SyntaxNodeContext, SyntaxPoint};

#[derive(Debug, Clone, PartialEq, Eq)]
struct AncestorContextQuery {
    buffer_id: u64,
    buffer_revision: u64,
    language_id: String,
    cursor_line: usize,
    cursor_column: usize,
}

struct LoadedGrammar {
    language_id: String,
    language: Language,
    _library: Option<libloading::Library>,
}

#[derive(Default)]
struct AncestorContextCache {
    loaded: Option<LoadedGrammar>,
    last_query: Option<AncestorContextQuery>,
    last_contexts: Vec<SyntaxNodeContext>,
}

thread_local! {
    static ANCESTOR_CONTEXT_CACHE: RefCell<AncestorContextCache> =
        RefCell::new(AncestorContextCache::default());
}

fn context_queries_enabled(language_id: &str) -> bool {
    !matches!(language_id, "sql" | "toml")
}

/// Returns named tree-sitter ancestor contexts for the provided cursor position.
pub fn ancestor_contexts_for_cursor(
    languages: &[LanguageConfiguration],
    language_id: Option<&str>,
    buffer_text: &str,
    buffer_id: u64,
    buffer_revision: u64,
    cursor_line: usize,
    cursor_column: usize,
) -> Vec<SyntaxNodeContext> {
    let Some(language_id) = language_id else {
        return Vec::new();
    };
    if buffer_text.is_empty() {
        return Vec::new();
    }
    if !context_queries_enabled(language_id) {
        return Vec::new();
    }
    let query = AncestorContextQuery {
        buffer_id,
        buffer_revision,
        language_id: language_id.to_owned(),
        cursor_line,
        cursor_column,
    };
    ANCESTOR_CONTEXT_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(contexts) = cache
            .last_query
            .as_ref()
            .filter(|cached| **cached == query)
            .map(|_| cache.last_contexts.clone())
        {
            return contexts;
        }
        let Some(config) = languages
            .iter()
            .find(|language| language.id() == language_id)
        else {
            return Vec::new();
        };
        if !ensure_loaded_language(&mut cache, config) {
            return Vec::new();
        }
        let Some(loaded) = cache.loaded.as_ref() else {
            return Vec::new();
        };
        let contexts =
            ancestor_contexts_from_text(&loaded.language, buffer_text, cursor_line, cursor_column);
        cache.last_query = Some(query);
        cache.last_contexts = contexts.clone();
        contexts
    })
}

/// Returns a single normalized source line.
pub fn buffer_line_text(
    buffer_text: &str,
    _buffer_id: u64,
    _buffer_revision: u64,
    line_index: usize,
) -> Option<String> {
    if buffer_text.is_empty() {
        return None;
    }
    buffer_text.lines().nth(line_index).map(str::to_owned)
}

fn ensure_loaded_language(
    cache: &mut AncestorContextCache,
    config: &LanguageConfiguration,
) -> bool {
    if cache
        .loaded
        .as_ref()
        .is_some_and(|loaded| loaded.language_id == config.id())
    {
        return true;
    }
    cache.loaded = load_language(config);
    cache.last_query = None;
    cache.loaded.is_some()
}

fn load_language(config: &LanguageConfiguration) -> Option<LoadedGrammar> {
    match config.loader() {
        LanguageLoader::Static {
            language_provider, ..
        } => Some(LoadedGrammar {
            language_id: config.id().to_owned(),
            language: language_provider(),
            _library: None,
        }),
        LanguageLoader::Grammar { grammar } => {
            let library_path = grammar.installed_library_path(&grammar_install_root());
            if !library_path.exists() {
                return None;
            }
            let library = unsafe {
                // SAFETY: Path comes from GrammarSource::installed_library_path under the
                // configured grammar install root for a tree-sitter grammar shared library.
                libloading::Library::new(&library_path)
            }
            .ok()?;
            let symbol_name = format!("{}\0", grammar.symbol_name());
            let symbol = unsafe {
                // SAFETY: Symbol name is the configured tree-sitter language constructor.
                library.get::<unsafe extern "C" fn() -> *const ()>(symbol_name.as_bytes())
            }
            .ok()?;
            let language_fn = unsafe {
                // SAFETY: Tree-sitter grammar libraries export LanguageFn-compatible constructors.
                LanguageFn::from_raw(*symbol)
            };
            Some(LoadedGrammar {
                language_id: config.id().to_owned(),
                language: Language::new(language_fn),
                _library: Some(library),
            })
        }
    }
}

fn ancestor_contexts_from_text(
    language: &Language,
    buffer_text: &str,
    cursor_line: usize,
    cursor_column: usize,
) -> Vec<SyntaxNodeContext> {
    let mut parser = Parser::new();
    if parser.set_language(language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(buffer_text, None) else {
        return Vec::new();
    };
    let point = tree_sitter::Point::new(cursor_line, cursor_column);
    let Some(mut node) = tree
        .root_node()
        .named_descendant_for_point_range(point, point)
    else {
        return Vec::new();
    };
    let mut contexts = Vec::new();
    loop {
        let start = node.start_position();
        let end = node.end_position();
        let parent = node.parent();
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
    contexts
}
