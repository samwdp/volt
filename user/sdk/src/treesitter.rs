use std::cell::RefCell;

use editor_buffer::{TextBuffer, TextPoint};
use editor_syntax::{LanguageConfiguration, SyntaxNodeContext, SyntaxParseSession, SyntaxRegistry};

#[derive(Debug, Clone, PartialEq, Eq)]
struct AncestorContextQuery {
    buffer_id: u64,
    buffer_revision: u64,
    language_id: String,
    cursor_line: usize,
    cursor_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AncestorContextBufferKey {
    buffer_id: u64,
    buffer_revision: u64,
}

#[derive(Default)]
struct AncestorContextCache {
    registry: Option<SyntaxRegistry>,
    last_buffer_key: Option<AncestorContextBufferKey>,
    last_buffer: Option<TextBuffer>,
    parse_session: Option<SyntaxParseSession>,
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
    let buffer_key = AncestorContextBufferKey {
        buffer_id,
        buffer_revision,
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
        ensure_cached_buffer(&mut cache, buffer_text, &buffer_key);
        let AncestorContextCache {
            registry,
            last_buffer,
            parse_session,
            ..
        } = &mut *cache;
        let Some(registry) = registry.as_mut() else {
            return Vec::new();
        };
        let Some(buffer) = last_buffer.as_ref() else {
            return Vec::new();
        };
        let contexts = registry
            .ancestor_contexts_for_language_with_parse_session(
                language_id,
                buffer,
                TextPoint::new(cursor_line, cursor_column),
                parse_session,
            )
            .unwrap_or_default();
        cache.last_query = Some(query);
        cache.last_contexts = contexts.clone();
        contexts
    })
}

/// Returns a single normalized source line using the same cached buffer backing tree-sitter
/// ancestor queries.
pub fn buffer_line_text(
    buffer_text: &str,
    buffer_id: u64,
    buffer_revision: u64,
    line_index: usize,
) -> Option<String> {
    if buffer_text.is_empty() {
        return None;
    }
    let buffer_key = AncestorContextBufferKey {
        buffer_id,
        buffer_revision,
    };
    ANCESTOR_CONTEXT_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        ensure_cached_buffer(&mut cache, buffer_text, &buffer_key);
        cache.last_buffer.as_ref()?.line(line_index)
    })
}

fn ensure_cached_buffer(
    cache: &mut AncestorContextCache,
    buffer_text: &str,
    buffer_key: &AncestorContextBufferKey,
) {
    if cache.last_buffer_key.as_ref() == Some(buffer_key) && cache.last_buffer.is_some() {
        return;
    }
    cache.last_buffer_key = Some(buffer_key.clone());
    cache.last_buffer = Some(TextBuffer::from_text(buffer_text));
    cache.parse_session = None;
    cache.last_query = None;
    cache.last_contexts.clear();
}
