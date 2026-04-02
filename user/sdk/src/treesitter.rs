use std::cell::RefCell;

use editor_buffer::{TextBuffer, TextPoint};
use editor_syntax::{LanguageConfiguration, SyntaxNodeContext, SyntaxRegistry};

#[derive(Debug, Clone, PartialEq, Eq)]
struct AncestorContextQuery {
    buffer_id: u64,
    buffer_revision: u64,
    language_id: String,
    cursor_line: usize,
    cursor_column: usize,
}

#[derive(Default)]
struct AncestorContextCache {
    registry: Option<SyntaxRegistry>,
    last_query: Option<AncestorContextQuery>,
    last_contexts: Vec<SyntaxNodeContext>,
}

thread_local! {
    static ANCESTOR_CONTEXT_CACHE: RefCell<AncestorContextCache> =
        RefCell::new(AncestorContextCache::default());
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
        if cache.registry.is_none() {
            let mut registry = SyntaxRegistry::new();
            if registry.register_all(languages.iter().cloned()).is_err() {
                return Vec::new();
            }
            cache.registry = Some(registry);
        }
        let Some(registry) = cache.registry.as_mut() else {
            return Vec::new();
        };
        let buffer = TextBuffer::from_text(buffer_text);
        let contexts = registry
            .ancestor_contexts_for_language(
                language_id,
                &buffer,
                TextPoint::new(cursor_line, cursor_column),
            )
            .unwrap_or_default();
        cache.last_query = Some(query);
        cache.last_contexts = contexts.clone();
        contexts
    })
}
