fn load_language(
    config: &LanguageConfiguration,
    install_root: &Path,
    query_asset_root: Option<&Path>,
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
            let query_source =
                append_query_source(highlight_query.to_owned(), config.extra_highlight_query());
            let query = compile_query_source(&language, config.id(), "highlight", &query_source)?;
            let (capture_names, capture_tokens) = intern_query_captures(&query, &capture_mappings);
            Ok(LoadedLanguage {
                _library: None,
                language,
                query,
                indent_query: DeferredQuery::from_source(
                    "indent",
                    config.extra_indent_query().map(str::to_owned),
                ),
                injections_query: DeferredQuery::from_source(
                    "injections",
                    config.extra_injections_query().map(str::to_owned),
                ),
                locals_query: DeferredQuery::from_source(
                    "locals",
                    config.extra_locals_query().map(str::to_owned),
                ),
                folds_query: DeferredQuery::from_source(
                    "folds",
                    config.extra_folds_query().map(str::to_owned),
                ),
                capture_names,
                capture_tokens,
            })
        }
        LanguageLoader::Grammar { grammar } => {
            let install_dir = install_root.to_path_buf();
            let library_path = grammar.installed_library_path(install_root);
            if !library_path.exists() {
                return Err(SyntaxError::GrammarNotInstalled {
                    language_id: config.id().to_owned(),
                    install_dir,
                });
            }

            let query_source = required_query_source(
                query_asset_root,
                config.id(),
                "highlights.scm",
                "highlight",
                config.extra_highlight_query(),
            )?;
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
            let query = compile_query_source(&language, config.id(), "highlight", &query_source)?;
            let (capture_names, capture_tokens) = intern_query_captures(&query, &capture_mappings);
            Ok(LoadedLanguage {
                _library: Some(ManuallyDrop::new(library)),
                language,
                query,
                indent_query: DeferredQuery::from_source(
                    "indent",
                    optional_query_source(
                        query_asset_root,
                        config.id(),
                        "indents.scm",
                        config.extra_indent_query(),
                    )?,
                ),
                injections_query: DeferredQuery::from_source(
                    "injections",
                    optional_query_source(
                        query_asset_root,
                        config.id(),
                        "injections.scm",
                        config.extra_injections_query(),
                    )?,
                ),
                locals_query: DeferredQuery::from_source(
                    "locals",
                    optional_query_source(
                        query_asset_root,
                        config.id(),
                        "locals.scm",
                        config.extra_locals_query(),
                    )?,
                ),
                folds_query: DeferredQuery::from_source(
                    "folds",
                    optional_query_source(
                        query_asset_root,
                        config.id(),
                        "folds.scm",
                        config.extra_folds_query(),
                    )?,
                ),
                capture_names,
                capture_tokens,
            })
        }
    }
}

fn intern_query_captures(
    query: &Query,
    capture_mappings: &BTreeMap<String, String>,
) -> (Vec<Arc<str>>, Vec<Arc<str>>) {
    let capture_names = query
        .capture_names()
        .iter()
        .copied()
        .map(Arc::<str>::from)
        .collect::<Vec<_>>();
    let capture_tokens = capture_names
        .iter()
        .map(|name| intern_theme_token(name, capture_mappings))
        .collect();
    (capture_names, capture_tokens)
}

fn intern_theme_token(capture_name: &str, mappings: &BTreeMap<String, String>) -> Arc<str> {
    mappings
        .get(capture_name)
        .map(|token| Arc::<str>::from(token.as_str()))
        .unwrap_or_else(|| Arc::from(format!("syntax.{capture_name}")))
}

fn append_query_source(mut source: String, extra_query: Option<&str>) -> String {
    if let Some(extra_query) = extra_query {
        if !source.ends_with('\n') && !source.is_empty() {
            source.push('\n');
        }
        source.push_str(extra_query);
    }
    source
}

fn compile_query_source(
    language: &Language,
    language_id: &str,
    query_kind: &str,
    source: &str,
) -> Result<Query, SyntaxError> {
    // Upstream queries sometimes use Vim's `\c` prefix for case-insensitive regexes.
    // tree-sitter's regex engine accepts the equivalent inline `(?i)` flag.
    let source = source.replace("\\\\c", "(?i)");
    Query::new(language, &source).map_err(|error| SyntaxError::InvalidQuery {
        language_id: language_id.to_owned(),
        query_kind: query_kind.to_owned(),
        message: error.to_string(),
    })
}

fn maybe_read_bundled_query_source(
    query_asset_root: Option<&Path>,
    language_id: &str,
    file_name: &str,
) -> Result<Option<String>, SyntaxError> {
    if let Some(asset_root) = query_asset_root
        && let Some(source) =
            resolve_bundled_query_source(asset_root, language_id, file_name, &mut Vec::new())?
    {
        return Ok(Some(source));
    }
    Ok(None)
}

fn required_query_source(
    query_asset_root: Option<&Path>,
    language_id: &str,
    file_name: &str,
    kind_label: &str,
    extra_query: Option<&str>,
) -> Result<String, SyntaxError> {
    optional_query_source(query_asset_root, language_id, file_name, extra_query)?.ok_or_else(|| {
        SyntaxError::Io {
            operation: format!("locate bundled {kind_label} query"),
            path: query_asset_root
                .map(|root| root.join(language_id).join(file_name))
                .unwrap_or_else(|| PathBuf::from(file_name)),
            message: format!("bundled {file_name} is missing for language `{language_id}`"),
        }
    })
}

fn optional_query_source(
    query_asset_root: Option<&Path>,
    language_id: &str,
    file_name: &str,
    extra_query: Option<&str>,
) -> Result<Option<String>, SyntaxError> {
    let source_from_file =
        maybe_read_bundled_query_source(query_asset_root, language_id, file_name)?;
    Ok(match (source_from_file, extra_query) {
        (Some(source), extra) => Some(append_query_source(source, extra)),
        (None, Some(extra)) => Some(extra.to_owned()),
        (None, None) => None,
    })
}

fn resolve_query_source_from_raw(
    raw_source: &str,
    query_path: &Path,
    query_asset_root: Option<&Path>,
    file_name: &str,
    inheritance_stack: &mut Vec<(String, String)>,
) -> Result<String, SyntaxError> {
    let (inherited_languages, body) = parse_query_inherits(raw_source);
    if inherited_languages.is_empty() {
        return Ok(body);
    }

    let Some(query_asset_root) = query_asset_root else {
        return Err(SyntaxError::Io {
            operation: "resolve inherited query".to_owned(),
            path: query_path.to_path_buf(),
            message:
                "query declares inherited languages but no bundled query asset root is configured"
                    .to_owned(),
        });
    };

    let mut resolved = String::new();
    for inherited_language in inherited_languages {
        let inherited_source = resolve_bundled_query_source(
            query_asset_root,
            &inherited_language,
            file_name,
            inheritance_stack,
        )?
        .ok_or_else(|| SyntaxError::Io {
            operation: "resolve inherited query".to_owned(),
            path: query_asset_root.join(&inherited_language).join(file_name),
            message: format!(
                "inherited query `{file_name}` for language `{inherited_language}` is missing"
            ),
        })?;
        if !resolved.is_empty() && !resolved.ends_with('\n') {
            resolved.push('\n');
        }
        resolved.push_str(&inherited_source);
    }
    if !body.is_empty() {
        if !resolved.is_empty() && !resolved.ends_with('\n') {
            resolved.push('\n');
        }
        resolved.push_str(&body);
    }
    Ok(resolved)
}

fn resolve_bundled_query_source(
    query_asset_root: &Path,
    language_id: &str,
    file_name: &str,
    inheritance_stack: &mut Vec<(String, String)>,
) -> Result<Option<String>, SyntaxError> {
    let query_path = query_asset_root.join(language_id).join(file_name);
    if !query_path.exists() {
        return Ok(None);
    }
    if inheritance_stack
        .iter()
        .any(|(id, file)| id == language_id && file.eq_ignore_ascii_case(file_name))
    {
        return Err(SyntaxError::Io {
            operation: "resolve inherited query".to_owned(),
            path: query_path,
            message: "cyclic query inheritance detected".to_owned(),
        });
    }

    let raw_source = fs::read_to_string(&query_path)
        .map_err(|error| io_error("read bundled query", &query_path, error))?;
    inheritance_stack.push((language_id.to_owned(), file_name.to_owned()));
    let resolved = resolve_query_source_from_raw(
        &raw_source,
        &query_path,
        Some(query_asset_root),
        file_name,
        inheritance_stack,
    )?;
    inheritance_stack.pop();
    Ok(Some(resolved))
}

fn parse_query_inherits(source: &str) -> (Vec<String>, String) {
    let mut inherited_languages = Vec::new();
    let mut body_lines = Vec::new();
    let had_trailing_newline = source.ends_with('\n');

    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(inherits) = trimmed.strip_prefix("; inherits:") {
            for language in inherits.split(',') {
                let language = language.trim();
                if !language.is_empty() && !inherited_languages.iter().any(|id| id == language) {
                    inherited_languages.push(language.to_owned());
                }
            }
            continue;
        }
        body_lines.push(line);
    }

    let mut body = body_lines.join("\n");
    if had_trailing_newline {
        body.push('\n');
    }
    (inherited_languages, body)
}

fn remove_legacy_grammar_install_directory(
    grammar: &GrammarSource,
    install_root: &Path,
) -> Result<(), SyntaxError> {
    let legacy_dir = grammar.legacy_install_directory(install_root);
    if legacy_dir.exists() {
        fs::remove_dir_all(&legacy_dir).map_err(|error| {
            io_error(
                "remove legacy grammar install directory",
                &legacy_dir,
                error,
            )
        })?;
    }
    Ok(())
}

/// Converts an editor [`TextPoint`] (character columns) into a tree-sitter [`Point`]
/// whose columns are measured in UTF-8 bytes.
///
/// Out-of-bounds coordinates are clamped to the nearest valid line/column in the
/// provided buffer before converting character columns into byte columns.
fn text_point_to_tree_sitter_point(buffer: &impl SyntaxText, point: TextPoint) -> Point {
    let max_line = buffer.line_count().saturating_sub(1);
    let line = point.line.min(max_line);
    let text = buffer.line(line).unwrap_or_default();
    let column = text.chars().take(point.column).map(char::len_utf8).sum();
    Point { row: line, column }
}

fn tree_sitter_column_to_char_column(line: &str, byte_column: usize) -> usize {
    let mut bytes = 0usize;
    let mut chars = 0usize;
    for character in line.chars() {
        if bytes >= byte_column {
            break;
        }
        bytes = bytes.saturating_add(character.len_utf8());
        chars = chars.saturating_add(1);
    }
    chars
}

fn desired_indent_for_loaded_language(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &impl SyntaxText,
    line_index: usize,
    indent_width: usize,
    mut parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<Option<usize>, SyntaxError> {
    let Some(indent_query) = loaded.indent_query.query(&loaded.language, language_id)? else {
        return Ok(None);
    };
    if line_index >= buffer.line_count() || indent_width == 0 {
        return Ok(Some(0));
    }

    let parse_result = parse_tree(language_id, loaded, buffer, parse_session.as_deref_mut())?;
    let tree = require_tree(
        language_id,
        &parse_result,
        parse_session_ref(parse_session.as_deref()),
    )?;
    let mut query_cursor = QueryCursor::new();
    query_cursor.set_match_limit(INDENT_QUERY_MATCH_LIMIT);
    query_cursor.set_point_range(
        Point {
            row: line_index,
            column: 0,
        }..Point {
            row: line_index.saturating_add(1),
            column: 0,
        },
    );

    let capture_names = indent_query.capture_names();
    let mut saw_capture = false;
    let mut begin_levels = 0usize;
    let mut branch_levels = 0usize;
    let mut dedent_levels = 0usize;
    let mut zero = false;
    let mut fallback_to_auto = false;
    let mut aligned_indent: Option<usize> = None;
    let mut progress_steps = 0usize;
    let mut progress_callback = |_: &tree_sitter::QueryCursorState| {
        progress_steps = progress_steps.saturating_add(1);
        if progress_steps > INDENT_QUERY_PROGRESS_LIMIT {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let query_options = QueryCursorOptions::new().progress_callback(&mut progress_callback);
    let mut matches = query_cursor.matches_with_options(
        indent_query,
        tree.root_node(),
        SyntaxTextProvider { buffer },
        query_options,
    );
    let mut aborted = false;
    loop {
        matches.advance();
        let Some(query_match) = matches.get() else {
            break;
        };
        if !general_predicates_match(
            indent_query,
            query_match.pattern_index,
            query_match.captures,
            buffer,
        ) {
            continue;
        }

        let properties = indent_query.property_settings(query_match.pattern_index);
        for capture in query_match.captures {
            let Some(capture_name) = capture_names.get(capture.index as usize).copied() else {
                continue;
            };
            saw_capture = true;
            match capture_name {
                "indent.begin" if indent_begin_applies(capture.node, line_index, properties) => {
                    begin_levels = begin_levels.saturating_add(1);
                }
                "indent.branch" if line_index == capture.node.start_position().row => {
                    branch_levels = branch_levels.saturating_add(1);
                }
                "indent.dedent"
                    if line_index > capture.node.start_position().row
                        && line_index <= capture.node.end_position().row =>
                {
                    dedent_levels = dedent_levels.saturating_add(1);
                }
                "indent.align" => {
                    if let Some(column) =
                        aligned_indent_column(capture.node, line_index, properties, buffer)
                    {
                        aligned_indent =
                            Some(aligned_indent.map_or(column, |current| current.max(column)));
                    }
                }
                "indent.zero" if line_intersects_node(capture.node, line_index) => {
                    zero = true;
                }
                "indent.ignore" | "indent.auto"
                    if line_intersects_node(capture.node, line_index) =>
                {
                    fallback_to_auto = true;
                }
                "indent.end" => {}
                _ => {}
            }
        }
    }
    drop(matches);
    if query_cursor.did_exceed_match_limit() || progress_steps > INDENT_QUERY_PROGRESS_LIMIT {
        aborted = true;
    }
    if aborted {
        return Ok(None);
    }

    if zero {
        return Ok(Some(0));
    }
    if fallback_to_auto || !saw_capture {
        return Ok(None);
    }

    let levels = begin_levels.saturating_sub(branch_levels.saturating_add(dedent_levels));
    let level_columns = levels.saturating_mul(indent_width);
    Ok(Some(
        aligned_indent.map_or(level_columns, |column| column.max(level_columns)),
    ))
}

fn indent_begin_applies(node: Node<'_>, line_index: usize, properties: &[QueryProperty]) -> bool {
    let start_line = node.start_position().row;
    let end_line = node.end_position().row;
    (line_index > start_line
        || (line_index == start_line
            && query_property_is_set(properties, "indent.start_at_same_line")))
        && line_index <= end_line
}

fn aligned_indent_column(
    node: Node<'_>,
    line_index: usize,
    properties: &[QueryProperty],
    buffer: &impl SyntaxText,
) -> Option<usize> {
    if line_index <= node.start_position().row || line_index > node.end_position().row {
        return None;
    }

    let open_delimiter = query_property_value(properties, "indent.open_delimiter")?;
    let close_delimiter = query_property_value(properties, "indent.close_delimiter");
    if line_index == node.end_position().row
        && close_delimiter
            .is_some_and(|token| current_line_starts_with_token(buffer, line_index, token))
    {
        return None;
    }

    let start_line = buffer.line(node.start_position().row)?;
    let start_column = tree_sitter_column_to_char_column(&start_line, node.start_position().column);
    let open_column = delimiter_column(&start_line, start_column, open_delimiter)?;
    first_content_column_after(
        &start_line,
        open_column.saturating_add(open_delimiter.chars().count()),
    )
    .filter(|column| {
        close_delimiter
            .map(|token| !line_starts_with_token_at_column(&start_line, *column, token))
            .unwrap_or(true)
    })
    .or(Some(open_column.saturating_add(1)))
}

fn line_intersects_node(node: Node<'_>, line_index: usize) -> bool {
    let start_line = node.start_position().row;
    let end_line = node.end_position().row;
    start_line <= line_index && line_index <= end_line
}

fn query_property_is_set(properties: &[QueryProperty], key: &str) -> bool {
    properties
        .iter()
        .any(|property| property.key.as_ref() == key)
}

fn query_property_value<'a>(properties: &'a [QueryProperty], key: &str) -> Option<&'a str> {
    properties
        .iter()
        .find(|property| property.key.as_ref() == key)
        .and_then(|property| property.value.as_deref())
}

/// Returns the string value of the first `#set!` property with the given key that is
/// associated with a specific capture index, falling back to any pattern-wide property
/// with the same key.
///
/// This covers both the simple form `(#set! key "value")` (no capture, pattern-wide)
/// and the capture-targeted form `(#set! @capture key "value")`.
///
/// Returns `None` if no matching property is found or the property has no string value.
pub fn query_capture_property_value<'q>(
    query: &'q Query,
    pattern_index: usize,
    capture_index: u32,
    key: &str,
) -> Option<&'q str> {
    let properties = query.property_settings(pattern_index);
    // Capture-targeted properties take precedence over pattern-wide ones.
    for property in properties {
        if property.key.as_ref() == key && property.capture_id == Some(capture_index as usize) {
            return property.value.as_deref();
        }
    }
    for property in properties {
        if property.key.as_ref() == key && property.capture_id.is_none() {
            return property.value.as_deref();
        }
    }
    None
}

fn delimiter_column(line: &str, start_column: usize, delimiter: &str) -> Option<usize> {
    let delimiter = delimiter.chars().next()?;
    for (column, character) in line.chars().enumerate().skip(start_column) {
        if character == delimiter {
            return Some(column);
        }
    }
    None
}

fn first_content_column_after(line: &str, start_column: usize) -> Option<usize> {
    line.chars()
        .enumerate()
        .skip(start_column)
        .find_map(|(column, character)| (!character.is_whitespace()).then_some(column))
}

fn current_line_starts_with_token(
    buffer: &impl SyntaxText,
    line_index: usize,
    token: &str,
) -> bool {
    let line = buffer.line(line_index).unwrap_or_default();
    line.trim_start().starts_with(token)
}

fn line_starts_with_token_at_column(line: &str, column: usize, token: &str) -> bool {
    let tail = line.chars().skip(column).collect::<String>();
    tail.starts_with(token)
}

fn general_predicates_match(
    query: &Query,
    pattern_index: usize,
    captures: &[tree_sitter::QueryCapture<'_>],
    buffer: &impl SyntaxText,
) -> bool {
    query
        .general_predicates(pattern_index)
        .iter()
        .all(|predicate| {
            evaluate_general_predicate(
                predicate.operator.as_ref(),
                &predicate.args,
                captures,
                buffer,
            )
        })
}

fn evaluate_general_predicate(
    operator: &str,
    args: &[QueryPredicateArg],
    captures: &[tree_sitter::QueryCapture<'_>],
    buffer: &impl SyntaxText,
) -> bool {
    match operator.trim_start_matches('#') {
        "kind-eq?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            args.iter().skip(1).any(|argument| match argument {
                QueryPredicateArg::String(kind) => node.kind() == kind.as_ref(),
                QueryPredicateArg::Capture(_) => false,
            })
        }
        "not-kind-eq?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            args.iter().skip(1).all(|argument| match argument {
                QueryPredicateArg::String(kind) => node.kind() != kind.as_ref(),
                QueryPredicateArg::Capture(_) => true,
            })
        }
        "has-parent?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            let Some(parent) = node.parent() else {
                return false;
            };
            args.iter().skip(1).any(|argument| match argument {
                QueryPredicateArg::String(kind) => parent.kind() == kind.as_ref(),
                QueryPredicateArg::Capture(_) => false,
            })
        }
        "not-has-parent?" => {
            // Symmetric with `has-parent?`: checks only the immediate parent.
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            let Some(parent) = node.parent() else {
                return true;
            };
            args.iter().skip(1).all(|argument| match argument {
                QueryPredicateArg::String(kind) => parent.kind() != kind.as_ref(),
                QueryPredicateArg::Capture(_) => true,
            })
        }
        "has-ancestor?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            let mut ancestor = node.parent();
            while let Some(current) = ancestor {
                for argument in args.iter().skip(1) {
                    if let QueryPredicateArg::String(kind) = argument
                        && current.kind() == kind.as_ref()
                    {
                        return true;
                    }
                }
                ancestor = current.parent();
            }
            false
        }
        "not-has-ancestor?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            let mut ancestor = node.parent();
            while let Some(current) = ancestor {
                for argument in args.iter().skip(1) {
                    if let QueryPredicateArg::String(kind) = argument
                        && current.kind() == kind.as_ref()
                    {
                        return false;
                    }
                }
                ancestor = current.parent();
            }
            true
        }
        "lua-match?" => {
            let Some(text) = predicate_capture_text(args.first(), captures, buffer) else {
                return false;
            };
            let Some(pattern) = args.get(1).and_then(|argument| match argument {
                QueryPredicateArg::String(pattern) => Some(pattern.as_ref()),
                QueryPredicateArg::Capture(_) => None,
            }) else {
                return false;
            };
            lua_pattern_matches(&text, pattern)
        }
        "not-lua-match?" => {
            let Some(text) = predicate_capture_text(args.first(), captures, buffer) else {
                return false;
            };
            let Some(pattern) = args.get(1).and_then(|argument| match argument {
                QueryPredicateArg::String(pattern) => Some(pattern.as_ref()),
                QueryPredicateArg::Capture(_) => None,
            }) else {
                return false;
            };
            !lua_pattern_matches(&text, pattern)
        }
        "contains?" => {
            let Some(text) = predicate_capture_text(args.first(), captures, buffer) else {
                return false;
            };
            args.iter().skip(1).any(|argument| match argument {
                QueryPredicateArg::String(needle) => text.contains(needle.as_ref()),
                QueryPredicateArg::Capture(_) => false,
            })
        }
        // Directives (ending in `!`) are metadata annotations, not match filters.
        // They do not cause a match to be rejected.
        op if op.ends_with('!') => true,
        // Unknown filter predicates are allowed through. We cannot evaluate them
        // here, so we avoid silently discarding matches that depend on them.
        _ => true,
    }
}

fn predicate_capture_node<'tree>(
    argument: Option<&QueryPredicateArg>,
    captures: &[tree_sitter::QueryCapture<'tree>],
) -> Option<Node<'tree>> {
    let QueryPredicateArg::Capture(capture_id) = argument? else {
        return None;
    };
    captures
        .iter()
        .find(|capture| capture.index == *capture_id)
        .map(|capture| capture.node)
}

fn predicate_capture_text(
    argument: Option<&QueryPredicateArg>,
    captures: &[tree_sitter::QueryCapture<'_>],
    buffer: &impl SyntaxText,
) -> Option<String> {
    let node = predicate_capture_node(argument, captures)?;
    let mut text = String::new();
    for chunk in buffer.byte_slice_chunks(node.byte_range()) {
        text.push_str(std::str::from_utf8(chunk).ok()?);
    }
    Some(text)
}

/// Minimal Lua 5.x pattern matcher sufficient for the patterns used in tree-sitter query
/// corpora.  Supported pattern items:
///
/// * `.`             — any character
/// * `%a`/`%A`       — letter / non-letter
/// * `%d`/`%D`       — digit / non-digit
/// * `%l`/`%L`       — lowercase / non-lowercase
/// * `%u`/`%U`       — uppercase / non-uppercase
/// * `%w`/`%W`       — alphanumeric / non-alphanumeric
/// * `%s`/`%S`       — whitespace / non-whitespace
/// * `%p`/`%P`       — punctuation / non-punctuation
/// * `%x`/`%X`       — hex digit / non-hex-digit
/// * `%c`/`%C`       — control char / non-control-char
/// * `%(` `%)` etc.  — escaped literal
/// * `[set]`/`[^set]`— character class (ranges a–z supported)
/// * `*` `+` `-` `?` — quantifiers on the preceding item
/// * `^`             — anchor at start
/// * `$`             — anchor at end
///
/// Captures (`(…)`) are parsed but their contents are otherwise ignored for the
/// match/no-match decision.
fn lua_pattern_matches(text: &str, pattern: &str) -> bool {
    let text_bytes = text.as_bytes();
    let pat_bytes = pattern.as_bytes();

    let (anchored, pat_start) = if pat_bytes.first() == Some(&b'^') {
        (true, 1)
    } else {
        (false, 0)
    };

    if anchored {
        lua_match_here(text_bytes, 0, pat_bytes, pat_start)
    } else {
        // Try matching at every starting position.
        for start in 0..=text_bytes.len() {
            if lua_match_here(text_bytes, start, pat_bytes, pat_start) {
                return true;
            }
        }
        false
    }
}

/// Try to match `pat[pi..]` against `text[ti..]`.
fn lua_match_here(text: &[u8], mut ti: usize, pat: &[u8], mut pi: usize) -> bool {
    loop {
        // End of pattern — success.
        if pi >= pat.len() {
            return true;
        }

        // `$` at end of pattern anchors to end of text.
        if pat[pi] == b'$' && pi + 1 == pat.len() {
            return ti == text.len();
        }

        // Opening capture group `(` — skip, we only need match/no-match.
        if pat[pi] == b'(' {
            pi += 1;
            continue;
        }
        // Closing capture group `)` — skip.
        if pat[pi] == b')' {
            pi += 1;
            continue;
        }

        // Determine how many bytes the current pattern item consumes (item_len) and
        // the end of the item in `pat` (item_end), then look ahead for a quantifier.
        let (item_end, item_len_in_pat) = lua_item_span(pat, pi);
        let quantifier = pat.get(item_end).copied();

        match quantifier {
            Some(b'*') => {
                // Match zero or more (greedy).
                let q_end = item_end + 1;
                let mut ti2 = ti;
                while ti2 < text.len() && lua_item_matches(text[ti2], pat, pi, item_end) {
                    ti2 += 1;
                }
                // Try longest first.
                loop {
                    if lua_match_here(text, ti2, pat, q_end) {
                        return true;
                    }
                    if ti2 == ti {
                        break;
                    }
                    ti2 -= 1;
                }
                return false;
            }
            Some(b'+') => {
                // One or more.
                if ti >= text.len() || !lua_item_matches(text[ti], pat, pi, item_end) {
                    return false;
                }
                ti += 1;
                let q_end = item_end + 1;
                let mut ti2 = ti;
                while ti2 < text.len() && lua_item_matches(text[ti2], pat, pi, item_end) {
                    ti2 += 1;
                }
                loop {
                    if lua_match_here(text, ti2, pat, q_end) {
                        return true;
                    }
                    if ti2 == ti {
                        break;
                    }
                    ti2 -= 1;
                }
                return false;
            }
            Some(b'-') => {
                // Lazy zero-or-more.
                let q_end = item_end + 1;
                loop {
                    if lua_match_here(text, ti, pat, q_end) {
                        return true;
                    }
                    if ti >= text.len() || !lua_item_matches(text[ti], pat, pi, item_end) {
                        return false;
                    }
                    ti += 1;
                }
            }
            Some(b'?') => {
                // Zero or one.
                let q_end = item_end + 1;
                if ti < text.len()
                    && lua_item_matches(text[ti], pat, pi, item_end)
                    && lua_match_here(text, ti + 1, pat, q_end)
                {
                    return true;
                }
                pi = q_end;
                // fall through (zero occurrences)
            }
            _ => {
                // No quantifier — match exactly one.
                if ti >= text.len() || !lua_item_matches(text[ti], pat, pi, item_end) {
                    return false;
                }
                ti += 1;
                pi = item_end;
                let _ = item_len_in_pat;
            }
        }
    }
}

/// Returns `(item_end, item_byte_len)` where `item_end` is the index in `pat` of the
/// first byte *after* the current pattern item that starts at `pi`.
fn lua_item_span(pat: &[u8], pi: usize) -> (usize, usize) {
    if pat[pi] == b'%' {
        // Escaped character: `%x` — always 2 bytes.
        (pi + 2, 2)
    } else if pat[pi] == b'[' {
        // Character class: `[…]` — find the closing `]`.
        let mut i = pi + 1;
        if i < pat.len() && pat[i] == b'^' {
            i += 1;
        }
        // A `]` immediately after `[` or `[^` is treated as a literal.
        if i < pat.len() && pat[i] == b']' {
            i += 1;
        }
        while i < pat.len() && pat[i] != b']' {
            if pat[i] == b'%' {
                i += 1; // skip the escaped char
            }
            i += 1;
        }
        (i + 1, i + 1 - pi) // include the closing `]`
    } else {
        (pi + 1, 1)
    }
}
