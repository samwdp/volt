use super::*;
use crate::{Diagnostic, DiagnosticSeverity, LanguageServerRegistry};
use editor_buffer::{TextPoint, TextRange};
use lsp_types::{
    NumberOrString, TextDocumentSyncKind, WorkDoneProgressParams,
    request::{CodeActionRequest, HoverRequest, Initialize, Request, SignatureHelpRequest},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::{Child, ChildStdin},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64},
    },
};

#[cfg(windows)]
fn temp_dir() -> PathBuf {
    use std::time::SystemTime;
    let unique = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("volt-fnm-lsp-{unique}"))
}

/// Drive-letter root on Windows, `/name` on Unix so `Path::starts_with` sees real components.
fn sample_root(name: &str) -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(r"P:\").join(name)
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/").join(name)
    }
}

#[test]
fn completion_parser_handles_lists_and_docs() {
    let response = json!({
        "isIncomplete": false,
        "items": [
            {
                "label": "println!",
                "kind": 3,
                "insertText": "println!",
                "detail": "macro_rules! println",
                "documentation": { "kind": "markdown", "value": "Prints to stdout." }
            }
        ]
    });
    let items = parse_completion_response("rust-analyzer", &response);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].label(), "println!");
    assert_eq!(items[0].kind(), Some(LspCompletionKind::Function));
    assert_eq!(items[0].documentation(), Some("Prints to stdout."));
    assert_eq!(items[0].edit_range(), None);
}

#[test]
fn completion_parser_prefers_text_edit_over_insert_text_and_keeps_range() {
    // csharp-ls / Roslyn style: typed "foo." then item replaces the "." with ".bar()".
    let response = json!([
        {
            "label": "bar",
            "kind": 2,
            "insertText": "bar",
            "textEdit": {
                "range": {
                    "start": { "line": 0, "character": 3 },
                    "end": { "line": 0, "character": 4 }
                },
                "newText": ".bar()"
            }
        }
    ]);
    let items = parse_completion_response("csharp-ls", &response);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].label(), "bar");
    assert_eq!(items[0].insert_text(), ".bar()");
    assert_eq!(
        items[0].edit_range(),
        Some(TextRange::new(TextPoint::new(0, 3), TextPoint::new(0, 4)))
    );
}

#[test]
fn completion_parser_reads_insert_replace_edit_replace_range() {
    let response = json!([
        {
            "label": "bar",
            "kind": 2,
            "textEdit": {
                "newText": "bar()",
                "insert": {
                    "start": { "line": 0, "character": 4 },
                    "end": { "line": 0, "character": 4 }
                },
                "replace": {
                    "start": { "line": 0, "character": 3 },
                    "end": { "line": 0, "character": 4 }
                }
            }
        }
    ]);
    let items = parse_completion_response("csharp-ls", &response);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].insert_text(), "bar()");
    assert_eq!(
        items[0].edit_range(),
        Some(TextRange::new(TextPoint::new(0, 3), TextPoint::new(0, 4)))
    );
}

#[test]
fn hover_parser_preserves_markdown_content() {
    let response = json!({
        "contents": {
            "kind": "markdown",
            "value": "```rust\nfn example()\n```\n\nSample docs"
        }
    });
    let hover = parse_hover_response("rust-analyzer", &response).expect("hover");
    assert_eq!(hover.server_id(), "rust-analyzer");
    assert!(hover.is_markdown());
    assert_eq!(hover.text(), "```rust\nfn example()\n```\n\nSample docs");
    assert_eq!(
        hover.lines(),
        &[
            "```rust".to_owned(),
            "fn example()".to_owned(),
            "```".to_owned(),
            String::new(),
            "Sample docs".to_owned()
        ]
    );
}

#[test]
fn hover_parser_formats_marked_string_language_blocks_as_markdown() {
    let response = json!({
        "contents": [
            {
                "language": "rust",
                "value": "fn main() {}"
            },
            "Runs the example."
        ]
    });
    let hover = parse_hover_response("rust-analyzer", &response).expect("hover");
    assert!(hover.is_markdown());
    assert_eq!(
        hover.text(),
        "```rust\nfn main() {}\n```\n\nRuns the example."
    );
    assert_eq!(hover.lines()[0], "```rust");
    assert_eq!(hover.lines()[1], "fn main() {}");
    assert_eq!(hover.lines()[4], "Runs the example.");
}

#[test]
fn hover_parser_keeps_plaintext_markup_plain() {
    let response = json!({
        "contents": {
            "kind": "plaintext",
            "value": "alpha\nbeta"
        }
    });
    let hover = parse_hover_response("rust-analyzer", &response).expect("hover");
    assert!(!hover.is_markdown());
    assert_eq!(hover.text(), "alpha\nbeta");
    assert_eq!(hover.lines(), &["alpha".to_owned(), "beta".to_owned()]);
}

#[test]
fn signature_help_parser_surfaces_active_parameter_and_docs() {
    let response = json!({
        "signatures": [
            {
                "label": "do_thing(value: String)",
                "documentation": "Fallback overload"
            },
            {
                "label": "do_thing(value: String, count: usize)",
                "documentation": {
                    "kind": "markdown",
                    "value": "Formats the value multiple times."
                },
                "parameters": [
                    {
                        "label": "value: String",
                        "documentation": "Value to format."
                    },
                    {
                        "label": "count: usize",
                        "documentation": {
                            "kind": "markdown",
                            "value": "Number of repetitions."
                        }
                    }
                ],
                "activeParameter": 1
            }
        ],
        "activeSignature": 1,
        "activeParameter": 0
    });
    let signature_help = parse_signature_help_response("rust-analyzer", &response)
        .expect("signature help response")
        .expect("signature help");
    assert_eq!(signature_help.server_id(), "rust-analyzer");
    let markdown = signature_help.markdown_text(Some("rust"));
    assert!(markdown.contains("**Signature 1/2**"));
    assert!(markdown.contains("**Signature 2/2 (active)**"));
    assert!(markdown.contains("```rust\ndo_thing(value: String)\n```"));
    assert!(markdown.contains("```rust\ndo_thing(value: String, count: usize)\n```"));
    assert!(!markdown.contains("**Parameter:**"));
    assert!(markdown.contains("Number of repetitions."));
    assert!(markdown.contains("Formats the value multiple times."));
    assert!(markdown.contains("Fallback overload"));
}

#[test]
fn signature_help_parser_supports_label_offsets() {
    let response = json!({
        "signatures": [
            {
                "label": "call(alpha, beta)",
                "parameters": [
                    {
                        "label": [5, 10]
                    },
                    {
                        "label": [12, 16]
                    }
                ]
            }
        ],
        "activeSignature": 0,
        "activeParameter": 1
    });
    let signature_help = parse_signature_help_response("rust-analyzer", &response)
        .expect("signature help response")
        .expect("signature help");
    assert!(
        signature_help
            .active_parameter_range()
            .is_some_and(|range| range.start == 12 && range.end == 16)
    );
}

#[test]
fn signature_help_active_parameter_range_supports_simple_labels() {
    let response = json!({
        "signatures": [
            {
                "label": "call(alpha, beta)",
                "parameters": [
                    {
                        "label": "alpha"
                    },
                    {
                        "label": "beta"
                    }
                ]
            }
        ],
        "activeSignature": 0,
        "activeParameter": 1
    });
    let signature_help = parse_signature_help_response("rust-analyzer", &response)
        .expect("signature help response")
        .expect("signature help");
    let range = signature_help
        .active_parameter_range()
        .expect("active parameter range");
    assert_eq!(range.label, "call(alpha, beta)");
    assert_eq!(range.start, 12);
    assert_eq!(range.end, 16);
}

#[test]
fn diagnostics_parser_maps_lsp_fields() {
    let params = json!({
        "uri": "file:///P:/volt/src/main.rs",
        "diagnostics": [
            {
                "range": {
                    "start": { "line": 1, "character": 2 },
                    "end": { "line": 1, "character": 5 }
                },
                "severity": 2,
                "source": "rust-analyzer",
                "message": "unused binding"
            }
        ]
    });
    let (path, diagnostics) = parse_publish_diagnostics(&params).expect("diagnostics");
    assert!(path.ends_with(Path::new("src").join("main.rs")));
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity(), DiagnosticSeverity::Warning);
    assert_eq!(diagnostics[0].message(), "unused binding");
}

#[cfg(windows)]
#[test]
fn file_uri_roundtrip_handles_windows_paths() {
    let path = PathBuf::from(r"P:\volt\src\main.rs");
    let uri = path_to_file_uri(&path);
    assert_eq!(file_uri_to_path(&uri), Some(path));
}

#[test]
fn workspace_configuration_settings_unwrap_matching_section() {
    assert_eq!(
        normalized_workspace_configuration_settings(
            Some("csharp"),
            Some(json!({
                "csharp": {
                    "solutionPathOverride": r"P:\volt\Volt.sln",
                }
            })),
        ),
        Some(json!({
            "solutionPathOverride": r"P:\volt\Volt.sln",
        }))
    );
}

#[test]
fn workspace_configuration_requests_return_matching_section_settings() {
    let workspace_configuration = SessionWorkspaceConfiguration {
        section: Some("csharp".to_owned()),
        base_settings: Some(json!({
            "formatting": {
                "organizeImports": true,
                "useTabs": false,
            }
        })),
        runtime_override: Some(json!({
            "solutionPathOverride": r"P:\volt\Volt.sln",
            "formatting": {
                "useTabs": true,
            }
        })),
    };

    let response = server_request_response(
        CSHARP_SERVER_ID,
        Some(Path::new(r"P:\volt")),
        Some(&Value::String("workspace/configuration".to_owned())),
        Some(&json!({
            "items": [
                { "section": "csharp" },
                { "section": "other" },
                {}
            ]
        })),
        Some(&workspace_configuration),
    );

    assert_eq!(
        response.result,
        json!([
            {
                "formatting": {
                    "organizeImports": true,
                    "useTabs": true,
                },
                "solutionPathOverride": r"P:\volt\Volt.sln",
            },
            null,
            null
        ])
    );
    assert!(response.notification.is_none());
    assert_eq!(
        workspace_configuration.did_change_configuration_payload(false),
        Some(json!({
            "csharp": {
                "formatting": {
                    "organizeImports": true,
                    "useTabs": true,
                },
                "solutionPathOverride": r"P:\volt\Volt.sln",
            }
        }))
    );
}

#[test]
fn workspace_configuration_requests_support_unsectioned_settings() {
    let workspace_configuration = SessionWorkspaceConfiguration {
        section: None,
        base_settings: Some(json!({
            "featureFlag": true,
        })),
        runtime_override: Some(json!({
            "featureFlag": false,
            "verbosity": "debug",
        })),
    };

    let response = server_request_response(
        "rust-analyzer",
        None,
        Some(&Value::String("workspace/configuration".to_owned())),
        Some(&json!({
            "items": [
                {},
                { "section": "other" }
            ]
        })),
        Some(&workspace_configuration),
    );

    assert_eq!(
        response.result,
        json!([
            {
                "featureFlag": false,
                "verbosity": "debug",
            },
            null
        ])
    );
    assert!(response.notification.is_none());
}

#[test]
fn server_request_response_turns_copilot_show_document_into_browser_action() {
    let response = server_request_response(
        COPILOT_SERVER_ID,
        Some(Path::new(r"P:\volt")),
        Some(&Value::String("window/showDocument".to_owned())),
        Some(&json!({
            "uri": "https://github.com/login/device"
        })),
        None,
    );

    assert_eq!(response.result, json!({ "success": true }));
    let notification = response.notification.expect("showDocument notification");
    assert_eq!(notification.server_id(), COPILOT_SERVER_ID);
    assert_eq!(notification.root(), Some(Path::new(r"P:\volt")));
    assert_eq!(
        notification.action(),
        Some(&LspNotificationAction::OpenBrowserPopup {
            url: "https://github.com/login/device".to_owned()
        })
    );
}

#[test]
fn copilot_status_notifications_offer_sign_in_action() {
    let notification = parse_copilot_status_notification(
        COPILOT_SERVER_ID,
        Some(Path::new(r"P:\volt")),
        &json!({
            "kind": "Error",
            "message": "Sign in to use GitHub Copilot."
        }),
    )
    .expect("copilot status notification");

    assert_eq!(notification.level(), LspNotificationLevel::Error);
    assert_eq!(notification.root(), Some(Path::new(r"P:\volt")));
    assert_eq!(
        notification.action(),
        Some(&LspNotificationAction::CopilotSignIn)
    );
}

#[test]
fn copilot_status_notifications_ignore_non_error_updates() {
    assert!(
        parse_copilot_status_notification(
            COPILOT_SERVER_ID,
            Some(Path::new(r"P:\volt")),
            &json!({
                "kind": "Normal",
                "message": "Getting code actions from Copilot."
            }),
        )
        .is_none()
    );
    assert!(
        parse_copilot_status_notification(
            COPILOT_SERVER_ID,
            Some(Path::new(r"P:\volt")),
            &json!({
                "kind": "Warning",
                "message": "Transient warning."
            }),
        )
        .is_none()
    );
}

#[test]
fn csharp_and_copilot_servers_receive_initialization_options() {
    assert_eq!(
        initialization_options_for_server(CSHARP_SERVER_ID, None),
        Some(json!({
            "experimental": {
                "csharp": {
                    "metadataUris": true,
                }
            }
        }))
    );
    assert_eq!(
        initialization_options_for_server(ROSLYN_LANGUAGE_SERVER_ID, None),
        Some(json!({
            "experimental": {
                "csharp": {
                    "metadataUris": true,
                }
            }
        }))
    );
    assert_eq!(
        initialization_options_for_server(COPILOT_SERVER_ID, None),
        Some(json!({
            "editorInfo": {
                "name": "Volt",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "editorPluginInfo": {
                "name": "Volt Copilot",
                "version": env!("CARGO_PKG_VERSION"),
            }
        }))
    );
}

#[test]
fn non_csharp_non_copilot_servers_do_not_receive_initialization_options() {
    assert_eq!(
        initialization_options_for_server("rust-analyzer", None),
        None
    );
    assert_eq!(initialization_options_for_server("marksman", None), None);
}

#[test]
fn runtime_initialization_options_merge_with_server_defaults() {
    assert_eq!(
        initialization_options_for_server(
            CSHARP_SERVER_ID,
            Some(&json!({
                "experimental": {
                    "csharp": {
                        "solutionStyle": true,
                    }
                }
            })),
        ),
        Some(json!({
            "experimental": {
                "csharp": {
                    "metadataUris": true,
                    "solutionStyle": true,
                }
            }
        }))
    );
    assert_eq!(
        initialization_options_for_server(
            "sqls",
            Some(&json!({
                "connectionConfig": {
                    "driver": "mssql",
                    "dataSourceName": "Data Source=example.database.windows.net;Initial Catalog=volt",
                }
            })),
        ),
        Some(json!({
            "connectionConfig": {
                "driver": "mssql",
                "dataSourceName": "Data Source=example.database.windows.net;Initial Catalog=volt",
            }
        }))
    );
}

#[test]
fn formatting_parser_maps_text_edits() {
    let response = json!([
        {
            "range": {
                "start": { "line": 2, "character": 4 },
                "end": { "line": 2, "character": 9 }
            },
            "newText": "value"
        }
    ]);

    let edits = parse_text_edit_response("rust-analyzer", "formatting", &response)
        .expect("formatting response")
        .expect("text edits");
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].range().start(), TextPoint::new(2, 4));
    assert_eq!(edits[0].range().end(), TextPoint::new(2, 9));
    assert_eq!(edits[0].new_text(), "value");
}

#[test]
fn definition_parser_supports_location_links() {
    let response = json!([
        {
            "targetUri": "file:///P:/volt/src/lib.rs",
            "targetRange": {
                "start": { "line": 10, "character": 0 },
                "end": { "line": 12, "character": 1 }
            },
            "targetSelectionRange": {
                "start": { "line": 11, "character": 4 },
                "end": { "line": 11, "character": 10 }
            }
        }
    ]);

    let locations = parse_definition_response("rust-analyzer", &response).expect("locations");
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].server_id(), "rust-analyzer");
    assert_eq!(locations[0].uri(), "file:///P:/volt/src/lib.rs");
    assert!(locations[0].is_file_path());
    assert!(
        locations[0]
            .file_path()
            .is_some_and(|path| path.ends_with(Path::new("src").join("lib.rs")))
    );
    assert!(
        locations[0]
            .path()
            .ends_with(Path::new("src").join("lib.rs"))
    );
    assert_eq!(locations[0].range().start(), TextPoint::new(11, 4));
    assert_eq!(locations[0].range().end(), TextPoint::new(11, 10));
}

#[test]
fn definition_parser_preserves_uri_backed_locations() {
    let response = json!([
        {
            "uri": "csharp:/metadata/Volt/Program",
            "range": {
                "start": { "line": 5, "character": 1 },
                "end": { "line": 5, "character": 12 }
            }
        }
    ]);

    let locations = parse_reference_response("csharp-ls", &response).expect("locations");
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].uri(), "csharp:/metadata/Volt/Program");
    assert!(!locations[0].is_file_path());
    assert_eq!(locations[0].file_path(), None);
    assert_eq!(
        locations[0].path().as_os_str().to_string_lossy(),
        "csharp:/metadata/Volt/Program"
    );
}

#[test]
fn location_sorting_deduplicates_reference_results() {
    let response = json!([
        {
            "uri": "file:///P:/volt/src/main.rs",
            "range": {
                "start": { "line": 7, "character": 3 },
                "end": { "line": 7, "character": 8 }
            }
        },
        {
            "uri": "file:///P:/volt/src/lib.rs",
            "range": {
                "start": { "line": 2, "character": 1 },
                "end": { "line": 2, "character": 6 }
            }
        },
        {
            "uri": "file:///P:/volt/src/main.rs",
            "range": {
                "start": { "line": 7, "character": 3 },
                "end": { "line": 7, "character": 8 }
            }
        }
    ]);

    let mut locations = parse_reference_response("rust-analyzer", &response).expect("locations");
    sort_locations(&mut locations);
    assert_eq!(locations.len(), 2);
    assert!(
        locations[0]
            .path()
            .ends_with(Path::new("src").join("lib.rs"))
    );
    assert!(
        locations[1]
            .path()
            .ends_with(Path::new("src").join("main.rs"))
    );
}

#[test]
fn code_action_parser_collects_active_file_edits() {
    let response = json!([
        {
            "title": "Fix unused import",
            "kind": "quickfix",
            "isPreferred": true,
            "edit": {
                "changes": {
                    "file:///P:/volt/src/main.rs": [
                        {
                            "range": {
                                "start": { "line": 3, "character": 0 },
                                "end": { "line": 4, "character": 0 }
                            },
                            "newText": ""
                        }
                    ]
                }
            }
        }
    ]);

    let actions = parse_code_action_response("rust-analyzer", &response).expect("code actions");
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].server_id(), "rust-analyzer");
    assert_eq!(actions[0].title(), "Fix unused import");
    assert_eq!(actions[0].kind(), Some("quickfix"));
    assert!(actions[0].is_preferred());
    assert_eq!(actions[0].document_edits().len(), 1);
    assert!(
        actions[0].document_edits()[0]
            .path()
            .ends_with(Path::new("src").join("main.rs"))
    );
    assert_eq!(actions[0].document_edits()[0].edits().len(), 1);
    assert_eq!(
        actions[0].document_edits()[0].edits()[0].range().start(),
        TextPoint::new(3, 0)
    );
}

#[test]
fn code_action_parser_tracks_command_and_resource_operations() {
    let response = json!([
        {
            "title": "Apply workspace fix",
            "kind": "quickfix",
            "disabled": {
                "reason": "build script output is stale"
            },
            "command": {
                "title": "Apply workspace edit",
                "command": "rust-analyzer.applySourceChange"
            },
            "edit": {
                "documentChanges": [
                    {
                        "textDocument": {
                            "uri": "file:///P:/volt/src/lib.rs",
                            "version": 3
                        },
                        "edits": [
                            {
                                "range": {
                                    "start": { "line": 0, "character": 0 },
                                    "end": { "line": 0, "character": 0 }
                                },
                                "newText": "use std::fmt;\n"
                            }
                        ]
                    },
                    {
                        "kind": "rename",
                        "oldUri": "file:///P:/volt/src/old.rs",
                        "newUri": "file:///P:/volt/src/new.rs"
                    }
                ]
            }
        },
        {
            "title": "Trigger organize imports",
            "command": "rust-analyzer.organizeImports"
        }
    ]);

    let actions = parse_code_action_response("rust-analyzer", &response).expect("code actions");
    assert_eq!(actions.len(), 2);
    assert_eq!(
        actions[0].disabled_reason(),
        Some("build script output is stale")
    );
    assert_eq!(
        actions[0].command_name(),
        Some("rust-analyzer.applySourceChange")
    );
    assert!(actions[0].has_resource_operations());
    assert_eq!(actions[0].document_edits().len(), 1);
    assert_eq!(
        actions[1].command_name(),
        Some("rust-analyzer.organizeImports")
    );
    assert!(actions[1].document_edits().is_empty());
}

#[test]
fn code_action_params_use_flattened_lsp_shape() {
    let path = Path::new("P:\\volt\\src\\main.ts");
    let range = TextRange::new(TextPoint::new(3, 4), TextPoint::new(3, 7));
    let diagnostics = vec![Diagnostic::new(
        "typescript",
        "Cannot find name `missingSymbol`.",
        DiagnosticSeverity::Error,
        range,
    )];
    let params = code_action_params(
        path,
        range,
        &diagnostics,
        WorkDoneProgressParams {
            work_done_token: Some(NumberOrString::String(
                "progress:textDocument/codeAction:test".to_owned(),
            )),
        },
    )
    .expect("code action params");
    let encoded = serde_json::to_value(params).expect("encoded params");
    let expected_uri = path_to_uri(path).expect("uri").to_string();

    assert_eq!(
        encoded
            .get("textDocument")
            .and_then(|text_document| text_document.get("uri"))
            .and_then(Value::as_str),
        Some(expected_uri.as_str())
    );
    assert_eq!(
        encoded.get("range"),
        Some(&json!({
            "start": { "line": 3, "character": 4 },
            "end": { "line": 3, "character": 7 },
        }))
    );
    assert_eq!(
        encoded
            .get("context")
            .and_then(|context| context.get("triggerKind"))
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        encoded
            .get("context")
            .and_then(|context| context.get("diagnostics"))
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        encoded
            .get("context")
            .and_then(|context| context.get("diagnostics"))
            .and_then(Value::as_array)
            .and_then(|diagnostics| diagnostics.first())
            .and_then(|diagnostic| diagnostic.get("severity"))
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        encoded.get("workDoneToken"),
        Some(&Value::String(
            "progress:textDocument/codeAction:test".to_owned(),
        ))
    );
    assert!(encoded.get("workDoneProgressParams").is_none());
    assert!(encoded.get("partialResultParams").is_none());
}

#[test]
fn point_requests_match_covering_diagnostics() {
    let point = TextPoint::new(4, 12);
    let range = TextRange::new(point, point);
    let diagnostic = TextRange::new(TextPoint::new(4, 0), TextPoint::new(4, 20));
    assert!(diagnostic_matches_request_range(diagnostic, range));
    assert!(!diagnostic_matches_request_range(
        TextRange::new(TextPoint::new(5, 0), TextPoint::new(5, 4)),
        range,
    ));
}

#[test]
fn transport_log_snapshot_is_bounded_and_tracks_revision() {
    let mut log = LspTransportLog::new(2);
    log.record(LspLogEntry::new(
        LspLogDirection::Event,
        "rust-analyzer",
        "started",
    ));
    log.record(LspLogEntry::new(
        LspLogDirection::Outgoing,
        "rust-analyzer",
        "{\"id\":1}",
    ));
    log.record(LspLogEntry::new(
        LspLogDirection::Incoming,
        "rust-analyzer",
        "{\"result\":1}",
    ));

    let snapshot = log.snapshot();
    assert_eq!(snapshot.revision(), 3);
    assert_eq!(snapshot.entries().len(), 2);
    assert_eq!(snapshot.entries()[0].direction(), LspLogDirection::Outgoing);
    assert_eq!(snapshot.entries()[1].direction(), LspLogDirection::Incoming);
}

#[test]
fn transport_message_redacts_sqls_connection_secrets() {
    let formatted = format_transport_message(&json!({
        "method": "workspace/didChangeConfiguration",
        "params": {
            "settings": {
                "sqls": {
                    "connections": [
                        {
                            "alias": "assetfusion",
                            "driver": "mssql",
                            "dataSourceName": "Data Source=assetfusion.database.windows.net;Password=secret;",
                            "password": "secret",
                            "passwd": "secret",
                        }
                    ]
                }
            }
        }
    }));

    assert!(formatted.contains("\"alias\": \"assetfusion\""));
    assert!(formatted.contains("\"driver\": \"mssql\""));
    assert!(formatted.contains("\"dataSourceName\": \"[redacted]\""));
    assert!(formatted.contains("\"password\": \"[redacted]\""));
    assert!(formatted.contains("\"passwd\": \"[redacted]\""));
    assert!(!formatted.contains("assetfusion.database.windows.net"));
    assert!(!formatted.contains("secret"));
}

#[test]
fn notification_log_snapshot_is_bounded_and_tracks_revision() {
    let mut log = LspNotificationLog::new(2);
    log.record(LspNotification {
        key: "session:rust-analyzer:global".to_owned(),
        server_id: "rust-analyzer".to_owned(),
        root: None,
        level: LspNotificationLevel::Info,
        title: "LSP · rust-analyzer".to_owned(),
        body_lines: vec!["Starting".to_owned()],
        progress: None,
        active: true,
        action: None,
    });
    log.record(LspNotification {
        key: "progress:rust-analyzer:token-1".to_owned(),
        server_id: "rust-analyzer".to_owned(),
        root: None,
        level: LspNotificationLevel::Info,
        title: "LSP · rust-analyzer".to_owned(),
        body_lines: vec!["Indexing".to_owned()],
        progress: Some(LspNotificationProgress::new(Some(25))),
        active: true,
        action: None,
    });
    log.record(LspNotification {
        key: "session:rust-analyzer:global".to_owned(),
        server_id: "rust-analyzer".to_owned(),
        root: None,
        level: LspNotificationLevel::Success,
        title: "LSP · rust-analyzer".to_owned(),
        body_lines: vec!["Ready".to_owned()],
        progress: None,
        active: false,
        action: None,
    });

    let snapshot = log.snapshot();
    assert_eq!(snapshot.revision(), 3);
    assert_eq!(snapshot.entries().len(), 2);
    assert_eq!(
        snapshot.entries()[0].notification().key(),
        "progress:rust-analyzer:token-1"
    );
    assert_eq!(
        snapshot.entries()[1].notification().level(),
        LspNotificationLevel::Success
    );
}

fn register_dummy_server(id: &str) -> LanguageServerRegistry {
    let mut registry = LanguageServerRegistry::new();
    registry
        .register(crate::LanguageServerSpec::new(
            id,
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register dummy server");
    registry
}

fn sample_diagnostic(message: &str) -> Diagnostic {
    Diagnostic::new(
        "rustc",
        message,
        DiagnosticSeverity::Error,
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 4)),
    )
}

#[test]
fn memory_session_publish_bumps_generation_and_dirty_path() {
    let path = PathBuf::from("src").join("main.rs");
    let manager = LspClientManager::new(register_dummy_server("rust-analyzer"));
    manager
        .attach_memory_session(
            "rust-analyzer",
            &path,
            vec![sample_diagnostic("cannot find value `missing`")],
        )
        .expect("attach memory session");

    assert_eq!(manager.diagnostics_generation(), 1);
    assert_eq!(
        manager.take_dirty_diagnostic_paths(),
        BTreeSet::from([path.clone()])
    );
    assert!(manager.take_dirty_diagnostic_paths().is_empty());

    manager
        .apply_published_diagnostics(&path, vec![sample_diagnostic("unused variable")])
        .expect("publish diagnostics");
    assert_eq!(manager.diagnostics_generation(), 2);
    assert_eq!(
        manager.take_dirty_diagnostic_paths(),
        BTreeSet::from([path.clone()])
    );
    assert_eq!(
        manager.diagnostics_for_path(&path)[0].message(),
        "unused variable"
    );
}

#[test]
fn memory_session_disconnect_bumps_generation_and_clears_path() {
    let path = PathBuf::from("src").join("lib.rs");
    let manager = LspClientManager::new(register_dummy_server("rust-analyzer"));
    manager
        .attach_memory_session(
            "rust-analyzer",
            &path,
            vec![sample_diagnostic("cannot find value `missing`")],
        )
        .expect("attach memory session");
    let _ = manager.take_dirty_diagnostic_paths();
    manager
        .disconnect_memory_sessions_for_path(&path)
        .expect("disconnect");

    assert_eq!(manager.diagnostics_generation(), 2);
    assert_eq!(
        manager.take_dirty_diagnostic_paths(),
        BTreeSet::from([path.clone()])
    );
    assert!(manager.diagnostics_for_path(&path).is_empty());
}

#[test]
fn log_and_notification_revision_skip_cloning_unchanged_snapshots() {
    let manager = LspClientManager::new(LanguageServerRegistry::new());
    assert_eq!(manager.log_revision(), 0);
    assert!(manager.log_snapshot_if_changed(0).is_none());
    manager.record_transport_log_event("rust-analyzer", "started");
    assert_eq!(manager.log_revision(), 1);
    let log = manager
        .log_snapshot_if_changed(0)
        .expect("log snapshot after revision move");
    assert_eq!(log.revision(), 1);
    assert!(manager.log_snapshot_if_changed(1).is_none());

    assert_eq!(manager.notification_revision(), 0);
    assert!(manager.notification_snapshot_if_changed(0).is_none());
    manager.record_show_message("rust-analyzer", "Indexing");
    assert_eq!(manager.notification_revision(), 1);
    let notifications = manager
        .notification_snapshot_if_changed(0)
        .expect("notification snapshot after revision move");
    assert_eq!(notifications.revision(), 1);
    assert!(manager.notification_snapshot_if_changed(1).is_none());
}

#[test]
fn two_memory_sessions_merge_and_sort_diagnostics_for_path() {
    let path = PathBuf::from("src").join("main.rs");
    let mut registry = LanguageServerRegistry::new();
    registry
        .register(crate::LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register rust-analyzer");
    registry
        .register(crate::LanguageServerSpec::new(
            "biome",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register biome");
    let manager = LspClientManager::new(registry);
    let later = Diagnostic::new(
        "rustc",
        "cannot find value `missing`",
        DiagnosticSeverity::Error,
        TextRange::new(TextPoint::new(4, 2), TextPoint::new(4, 9)),
    );
    let earlier = Diagnostic::new(
        "biome",
        "formatting suggestion",
        DiagnosticSeverity::Information,
        TextRange::new(TextPoint::new(1, 0), TextPoint::new(1, 6)),
    );
    manager
        .attach_memory_session("rust-analyzer", &path, vec![later.clone()])
        .expect("attach rust-analyzer");
    manager
        .attach_memory_session("biome", &path, vec![earlier.clone()])
        .expect("attach biome");

    let merged = manager.diagnostics_for_path(&path);
    assert_eq!(merged, vec![earlier, later]);
}

#[test]
fn progress_notifications_update_existing_track() {
    let begin = json!({
        "token": "rust-analyzer/index",
        "value": {
            "kind": "begin",
            "title": "Indexing",
            "message": "Scanning workspace",
            "percentage": 12
        }
    });
    let report = json!({
        "token": "rust-analyzer/index",
        "value": {
            "kind": "report",
            "message": "Building symbol graph",
            "percentage": 58
        }
    });
    let end = json!({
        "token": "rust-analyzer/index",
        "value": {
            "kind": "end",
            "message": "Indexed workspace"
        }
    });
    let mut tracks = BTreeMap::new();

    let begin = parse_progress_notification("rust-analyzer", None, &begin, &mut tracks)
        .expect("begin progress notification");
    assert!(begin.active());
    assert_eq!(begin.body_lines(), ["Indexing", "Scanning workspace"]);
    assert_eq!(
        begin
            .progress()
            .and_then(LspNotificationProgress::percentage),
        Some(12)
    );

    let report = parse_progress_notification("rust-analyzer", None, &report, &mut tracks)
        .expect("report progress notification");
    assert!(report.active());
    assert_eq!(report.body_lines(), ["Indexing", "Building symbol graph"]);
    assert_eq!(
        report
            .progress()
            .and_then(LspNotificationProgress::percentage),
        Some(58)
    );

    let end = parse_progress_notification("rust-analyzer", None, &end, &mut tracks)
        .expect("end progress notification");
    assert!(!end.active());
    assert_eq!(
        end.progress().and_then(LspNotificationProgress::percentage),
        Some(58)
    );
    assert_eq!(end.body_lines(), ["Indexing", "Indexed workspace"]);
    assert!(tracks.is_empty());
}

#[test]
fn client_capabilities_enable_window_work_done_progress_and_show_document() {
    let capabilities = client_capabilities().expect("client capabilities");
    assert_eq!(
        capabilities
            .window
            .as_ref()
            .and_then(|window| window.work_done_progress),
        Some(true)
    );
    assert_eq!(
        capabilities
            .window
            .and_then(|window| window.show_document)
            .map(|show_document| show_document.support),
        Some(true)
    );
}

#[test]
fn work_done_progress_params_generate_unique_tokens() {
    let next_progress_token = std::sync::atomic::AtomicU64::new(1);
    let hover = work_done_progress_params(&next_progress_token, HoverRequest::METHOD);
    let signature = work_done_progress_params(&next_progress_token, SignatureHelpRequest::METHOD);

    assert_eq!(
        hover.work_done_token,
        Some(lsp_types::NumberOrString::String(format!(
            "progress:{}:1",
            HoverRequest::METHOD
        )))
    );
    assert_eq!(
        signature.work_done_token,
        Some(lsp_types::NumberOrString::String(format!(
            "progress:{}:2",
            SignatureHelpRequest::METHOD
        )))
    );
}

#[test]
fn initialize_request_timeout_is_extended() {
    assert_eq!(
        request_timeout_for_method(Initialize::METHOD),
        INITIALIZE_REQUEST_TIMEOUT
    );
    assert_eq!(
        request_timeout_for_method(CodeActionRequest::METHOD),
        CODE_ACTION_REQUEST_TIMEOUT
    );
    assert_eq!(
        request_timeout_for_method(HoverRequest::METHOD),
        REQUEST_TIMEOUT
    );
}

#[test]
fn session_labels_ignore_stale_tracked_session_keys() {
    let manager = LspClientManager::new(LanguageServerRegistry::new());
    let path = PathBuf::from("src\\main.rs");
    let mut tracked = TrackedBufferState {
        revision: 1,
        version: 1,
        ..TrackedBufferState::default()
    };
    tracked
        .sessions
        .insert(SessionKey::new("rust-analyzer", None));
    manager
        .state
        .lock()
        .expect("state lock")
        .tracked_buffers
        .insert(path.clone(), tracked);

    assert!(manager.session_labels_for_path(&path).is_empty());
    assert!(!manager.has_live_sessions_for_path(&path));
}

#[test]
fn sync_buffer_preserves_manually_started_default_disabled_sessions() {
    let path = PathBuf::from("src").join("main.rs");
    let mut registry = LanguageServerRegistry::new();
    registry
        .register(crate::LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register rust analyzer");
    registry
        .register(
            crate::LanguageServerSpec::new(
                COPILOT_SERVER_ID,
                "rust",
                ["rs"],
                "dummy-lsp",
                std::iter::empty::<&str>(),
            )
            .with_enabled_by_default(false),
        )
        .expect("register copilot");
    let manager = LspClientManager::new(registry);
    let rust_session = test_session_handle("rust-analyzer", &path, BTreeMap::new());
    let copilot_session = test_session_handle(COPILOT_SERVER_ID, &path, BTreeMap::new());

    {
        let mut state = manager.state.lock().expect("state lock");
        state
            .sessions
            .insert(rust_session.key.clone(), Arc::clone(&rust_session));
        state
            .sessions
            .insert(copilot_session.key.clone(), Arc::clone(&copilot_session));
        let mut tracked = TrackedBufferState {
            revision: 1,
            version: 1,
            ..TrackedBufferState::default()
        };
        tracked.sessions.insert(rust_session.key.clone());
        tracked.sessions.insert(copilot_session.key.clone());
        state.tracked_buffers.insert(path.clone(), tracked);
    }

    let labels = manager
        .sync_buffer(&path, "fn main() {}".to_owned(), 2, None)
        .expect("sync buffer");
    assert_eq!(
        labels,
        vec![COPILOT_SERVER_ID.to_owned(), "rust-analyzer".to_owned()]
    );
    assert_eq!(
        manager.session_labels_for_path(&path),
        vec![COPILOT_SERVER_ID.to_owned(), "rust-analyzer".to_owned()]
    );
}

fn spawn_inert_child() -> (Child, ChildStdin) {
    super::spawn_inert_child().expect("spawn inert child")
}

fn test_session_handle(
    server_id: &str,
    path: &Path,
    diagnostics_by_path: BTreeMap<PathBuf, Vec<Diagnostic>>,
) -> Arc<LspSessionHandle> {
    test_session_handle_in_workspace(server_id, path, None, diagnostics_by_path)
}

fn test_session_handle_in_workspace(
    server_id: &str,
    path: &Path,
    workspace_root: Option<&Path>,
    diagnostics_by_path: BTreeMap<PathBuf, Vec<Diagnostic>>,
) -> Arc<LspSessionHandle> {
    let mut registry = LanguageServerRegistry::new();
    registry
        .register(crate::LanguageServerSpec::new(
            server_id,
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register test server");
    let session = registry
        .prepare_session_for_path(server_id, path, workspace_root)
        .expect("prepare test session");
    let workspace_configuration = Arc::new(Mutex::new(SessionWorkspaceConfiguration::new(
        &session, None,
    )));
    let (child, writer) = spawn_inert_child();
    Arc::new(LspSessionHandle {
        key: SessionKey::new(server_id, session.root().map(PathBuf::as_path)),
        session,
        child: Mutex::new(child),
        writer: Arc::new(Mutex::new(writer)),
        pending: Arc::new(Mutex::new(BTreeMap::new())),
        diagnostics: Arc::new(Mutex::new(diagnostics_by_path)),
        open_documents: Mutex::new(BTreeMap::new()),
        text_document_sync_kind: Mutex::new(TextDocumentSyncKind::FULL),
        workspace_configuration,
        initialization_options: None,
        transport_log: Arc::new(Mutex::new(LspTransportLog::new(TRANSPORT_LOG_MAX_ENTRIES))),
        next_request_id: AtomicU64::new(1),
        next_progress_token: AtomicU64::new(1),
        disconnected: Arc::new(AtomicBool::new(false)),
        #[cfg(test)]
        fail_next_send: AtomicBool::new(false),
        needs_full_document: Mutex::new(BTreeSet::new()),
        completion_resolve_supported: AtomicBool::new(false),
    })
}

#[test]
fn workspace_diagnostics_collect_entries_across_live_sessions() {
    let manager = LspClientManager::new(LanguageServerRegistry::new());
    let lib_path = PathBuf::from("src").join("lib.rs");
    let main_path = PathBuf::from("src").join("main.rs");
    let error = Diagnostic::new(
        "rustc",
        "cannot find value `missing` in this scope",
        DiagnosticSeverity::Error,
        TextRange::new(TextPoint::new(4, 2), TextPoint::new(4, 9)),
    );
    let warning = Diagnostic::new(
        "rustc",
        "unused variable: `value`",
        DiagnosticSeverity::Warning,
        TextRange::new(TextPoint::new(7, 4), TextPoint::new(7, 9)),
    );
    let info = Diagnostic::new(
        "biome",
        "formatting suggestion",
        DiagnosticSeverity::Information,
        TextRange::new(TextPoint::new(1, 0), TextPoint::new(1, 6)),
    );

    let rust_session = test_session_handle(
        "rust-analyzer",
        &lib_path,
        BTreeMap::from([
            (lib_path.clone(), vec![error.clone()]),
            (main_path.clone(), vec![warning.clone()]),
        ]),
    );
    let biome_session = test_session_handle(
        "biome",
        &main_path,
        BTreeMap::from([(main_path.clone(), vec![info.clone()])]),
    );

    {
        let mut state = manager.state.lock().expect("state lock");
        state
            .sessions
            .insert(rust_session.key.clone(), Arc::clone(&rust_session));
        state
            .sessions
            .insert(biome_session.key.clone(), Arc::clone(&biome_session));
    }

    let diagnostics = manager.workspace_diagnostics();
    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics.iter().any(|entry| {
        entry.server_id() == "rust-analyzer"
            && entry.path() == lib_path.as_path()
            && entry.diagnostic() == &error
    }));
    assert!(diagnostics.iter().any(|entry| {
        entry.server_id() == "rust-analyzer"
            && entry.path() == main_path.as_path()
            && entry.diagnostic() == &warning
    }));
    assert!(diagnostics.iter().any(|entry| {
        entry.server_id() == "biome"
            && entry.path() == main_path.as_path()
            && entry.diagnostic() == &info
    }));
}

#[test]
fn sync_buffer_reopens_document_for_restarted_session_with_same_key() {
    let path = PathBuf::from("src").join("main.rs");
    let mut registry = LanguageServerRegistry::new();
    registry
        .register(crate::LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register rust analyzer");
    let manager = LspClientManager::new(registry);
    let old_session = test_session_handle("rust-analyzer", &path, BTreeMap::new());
    let new_session = test_session_handle("rust-analyzer", &path, BTreeMap::new());

    old_session
        .open_documents
        .lock()
        .expect("open documents lock")
        .insert(path.clone(), "fn main() {}".to_owned());

    {
        let mut state = manager.state.lock().expect("state lock");
        state
            .sessions
            .insert(old_session.key.clone(), Arc::clone(&old_session));
        let mut tracked = TrackedBufferState {
            revision: 1,
            version: 1,
            ..TrackedBufferState::default()
        };
        tracked.sessions.insert(old_session.key.clone());
        state.tracked_buffers.insert(path.clone(), tracked);

        state
            .sessions
            .insert(new_session.key.clone(), Arc::clone(&new_session));
    }

    manager
        .sync_buffer(&path, "fn main() {}".to_owned(), 2, None)
        .expect("sync buffer");

    assert!(new_session.has_open_document(&path));
}

#[test]
fn sync_buffer_reuses_one_session_across_files_in_same_workspace() {
    let root = PathBuf::from("P:\\workspace");
    let first = root.join("src").join("a.rs");
    let second = root.join("src").join("b.rs");
    let mut registry = LanguageServerRegistry::new();
    registry
        .register(crate::LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register rust analyzer");
    let manager = LspClientManager::new(registry);
    let session = test_session_handle_in_workspace(
        "rust-analyzer",
        &first,
        Some(root.as_path()),
        BTreeMap::new(),
    );
    let session_key = session.key.clone();
    {
        let mut state = manager.state.lock().expect("state lock");
        state
            .sessions
            .insert(session_key.clone(), Arc::clone(&session));
    }

    manager
        .sync_buffer(&first, "fn a() {}".to_owned(), 1, Some(root.as_path()))
        .expect("sync first");
    manager
        .sync_buffer(&second, "fn b() {}".to_owned(), 1, Some(root.as_path()))
        .expect("sync second");

    let state = manager.state.lock().expect("state lock");
    assert_eq!(state.sessions.len(), 1);
    assert!(state.sessions.contains_key(&session_key));
    assert!(state.tracked_buffers.contains_key(&first));
    assert!(state.tracked_buffers.contains_key(&second));
}

#[test]
fn close_buffer_keeps_session_alive_for_next_file() {
    let root = PathBuf::from("P:\\workspace");
    let first = root.join("src").join("a.rs");
    let second = root.join("src").join("b.rs");
    let mut registry = LanguageServerRegistry::new();
    registry
        .register(crate::LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register rust analyzer");
    let manager = LspClientManager::new(registry);
    let session = test_session_handle_in_workspace(
        "rust-analyzer",
        &first,
        Some(root.as_path()),
        BTreeMap::new(),
    );
    let session_key = session.key.clone();
    {
        let mut state = manager.state.lock().expect("state lock");
        state
            .sessions
            .insert(session_key.clone(), Arc::clone(&session));
    }

    manager
        .sync_buffer(&first, "fn a() {}".to_owned(), 1, Some(root.as_path()))
        .expect("sync first");
    manager.close_buffer(&first).expect("close first");
    assert_eq!(
        manager.state.lock().expect("state lock").sessions.len(),
        1,
        "document close must not kill the language server"
    );

    manager
        .sync_buffer(&second, "fn b() {}".to_owned(), 1, Some(root.as_path()))
        .expect("sync second");
    let state = manager.state.lock().expect("state lock");
    assert_eq!(state.sessions.len(), 1);
    assert!(state.sessions.contains_key(&session_key));
    assert!(!state.tracked_buffers.contains_key(&first));
    assert!(state.tracked_buffers.contains_key(&second));
}

#[test]
fn stop_buffer_shuts_down_session() {
    let root = PathBuf::from("P:\\workspace");
    let path = root.join("src").join("a.rs");
    let mut registry = LanguageServerRegistry::new();
    registry
        .register(crate::LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register rust analyzer");
    let manager = LspClientManager::new(registry);
    let session = test_session_handle_in_workspace(
        "rust-analyzer",
        &path,
        Some(root.as_path()),
        BTreeMap::new(),
    );
    let session_key = session.key.clone();
    {
        let mut state = manager.state.lock().expect("state lock");
        state.sessions.insert(session_key, Arc::clone(&session));
    }

    manager
        .sync_buffer(&path, "fn a() {}".to_owned(), 1, Some(root.as_path()))
        .expect("sync");
    manager.stop_buffer(&path).expect("stop");
    assert!(
        manager
            .state
            .lock()
            .expect("state lock")
            .sessions
            .is_empty()
    );
}

#[cfg(windows)]
#[test]
fn session_key_normalizes_windows_root_casing() {
    let upper = SessionKey::new("ols", Some(Path::new(r"P:\odinpong")));
    let lower = SessionKey::new("ols", Some(Path::new(r"p:\odinpong")));
    assert_eq!(upper, lower);
}

fn attach_session(manager: &LspClientManager, session: &Arc<LspSessionHandle>) {
    manager
        .state
        .lock()
        .expect("state lock")
        .sessions
        .insert(session.key.clone(), Arc::clone(session));
}

fn incremental_test_session(server_id: &str, path: &Path) -> Arc<LspSessionHandle> {
    let session = test_session_handle(server_id, path, BTreeMap::new());
    *session
        .text_document_sync_kind
        .lock()
        .expect("text document sync kind lock") = TextDocumentSyncKind::INCREMENTAL;
    session
}

fn last_notification_params(session: &LspSessionHandle, method: &str) -> Option<Value> {
    let log = session.transport_log.lock().expect("transport log");
    log.snapshot()
        .entries()
        .iter()
        .rev()
        .filter(|entry| entry.direction() == LspLogDirection::Outgoing)
        .filter_map(|entry| serde_json::from_str::<Value>(entry.body()).ok())
        .find(|message| message.get("method").and_then(Value::as_str) == Some(method))
        .and_then(|message| message.get("params").cloned())
}

fn last_did_change_content_changes(session: &LspSessionHandle) -> Vec<Value> {
    last_notification_params(session, "textDocument/didChange")
        .and_then(|params| params.get("contentChanges").cloned())
        .and_then(|changes| changes.as_array().cloned())
        .unwrap_or_default()
}

fn register_test_rust_analyzer(registry: &mut LanguageServerRegistry) {
    registry
        .register(crate::LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register rust analyzer");
}

fn manager_with_incremental_session(path: &Path) -> (LspClientManager, Arc<LspSessionHandle>) {
    let mut registry = LanguageServerRegistry::new();
    register_test_rust_analyzer(&mut registry);
    let manager = LspClientManager::new(registry);
    let session = incremental_test_session("rust-analyzer", path);
    attach_session(&manager, &session);
    (manager, session)
}

#[test]
fn incremental_did_change_emits_one_event_per_contiguous_edit() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let mut buffer = open_buffer(&manager, &path, "hello");
    let open_revision = buffer.revision();
    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("a");
    buffer.insert_text("b");
    sync_buffer_edits(&manager, &path, &buffer, open_revision, None);

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].get("text").and_then(Value::as_str), Some("a"));
    assert_eq!(changes[1].get("text").and_then(Value::as_str), Some("b"));
    assert_eq!(
        changes[1].get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 6 },
            "end": { "line": 0, "character": 6 },
        }))
    );
}

#[test]
fn incremental_did_change_sends_only_inserted_character() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);

    let mut buffer = editor_buffer::TextBuffer::from_text("hello");
    let open_revision = buffer.revision();
    manager
        .sync_buffer(&path, buffer.text(), open_revision, None)
        .expect("didOpen");

    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    let edits = buffer
        .edits_since(open_revision)
        .expect("contiguous insert chain");
    manager
        .sync_buffer_with_edits(
            &path,
            buffer.text(),
            buffer.revision(),
            None,
            Some(edits.as_slice()),
        )
        .expect("didChange");

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    let change = &changes[0];
    assert_eq!(
        change.get("text").and_then(Value::as_str),
        Some("!"),
        "incremental didChange must send only the inserted character"
    );
    assert_eq!(
        change.get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 5 },
            "end": { "line": 0, "character": 5 },
        }))
    );
    assert!(change.get("rangeLength").is_none() || change.get("rangeLength") == Some(&Value::Null));
}

fn last_did_open_text(session: &LspSessionHandle) -> Option<String> {
    last_notification_params(session, "textDocument/didOpen").and_then(|params| {
        params
            .get("textDocument")
            .and_then(|text_document| text_document.get("text"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    })
}

fn open_buffer(manager: &LspClientManager, path: &Path, text: &str) -> editor_buffer::TextBuffer {
    let buffer = editor_buffer::TextBuffer::from_text(text);
    manager
        .sync_buffer(path, buffer.text(), buffer.revision(), None)
        .expect("didOpen");
    buffer
}

fn sync_buffer_edits(
    manager: &LspClientManager,
    path: &Path,
    buffer: &editor_buffer::TextBuffer,
    from_revision: u64,
    edits: Option<&[editor_buffer::TextEdit]>,
) {
    let chain = edits
        .map(Vec::from)
        .or_else(|| buffer.edits_since(from_revision));
    manager
        .sync_buffer_with_edits(
            path,
            buffer.text(),
            buffer.revision(),
            None,
            chain.as_deref(),
        )
        .expect("didChange");
}

#[test]
fn incremental_did_change_includes_newline_in_range_and_text() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let mut buffer = open_buffer(&manager, &path, "ab");
    let open_revision = buffer.revision();
    buffer.set_cursor(TextPoint::new(0, 1));
    buffer.insert_text("\n");
    sync_buffer_edits(&manager, &path, &buffer, open_revision, None);

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].get("text").and_then(Value::as_str), Some("\n"));
    assert_eq!(
        changes[0].get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 1 },
            "end": { "line": 0, "character": 1 },
        }))
    );

    let after_newline = buffer.revision();
    buffer.set_cursor(TextPoint::new(1, 0));
    buffer.insert_text("x");
    sync_buffer_edits(&manager, &path, &buffer, after_newline, None);
    let next = last_did_change_content_changes(&session);
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].get("text").and_then(Value::as_str), Some("x"));
    assert_eq!(
        next[0].get("range"),
        Some(&json!({
            "start": { "line": 1, "character": 0 },
            "end": { "line": 1, "character": 0 },
        }))
    );
}

#[test]
fn incremental_did_change_sends_empty_text_for_range_delete() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let mut buffer = open_buffer(&manager, &path, "hello");
    let open_revision = buffer.revision();
    buffer.delete(TextRange::new(TextPoint::new(0, 1), TextPoint::new(0, 4)));
    sync_buffer_edits(&manager, &path, &buffer, open_revision, None);

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].get("text").and_then(Value::as_str), Some(""));
    assert_eq!(
        changes[0].get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 1 },
            "end": { "line": 0, "character": 4 },
        }))
    );
}

#[test]
fn incremental_did_change_replace_covers_old_range_and_new_text() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let mut buffer = open_buffer(&manager, &path, "hello");
    let open_revision = buffer.revision();
    buffer.replace(
        TextRange::new(TextPoint::new(0, 1), TextPoint::new(0, 4)),
        "ipp",
    );
    sync_buffer_edits(&manager, &path, &buffer, open_revision, None);

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].get("text").and_then(Value::as_str), Some("ipp"));
    assert_eq!(
        changes[0].get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 1 },
            "end": { "line": 0, "character": 4 },
        }))
    );
}

#[test]
fn incremental_did_change_uses_utf16_columns_on_emoji_line() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let mut buffer = open_buffer(&manager, &path, "a😀b");
    let open_revision = buffer.revision();
    buffer.set_cursor(TextPoint::new(0, 2));
    buffer.insert_text("x");
    sync_buffer_edits(&manager, &path, &buffer, open_revision, None);

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].get("text").and_then(Value::as_str), Some("x"));
    assert_eq!(
        changes[0].get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 3 },
            "end": { "line": 0, "character": 3 },
        })),
        "😀 is one TextPoint column and two UTF-16 code units"
    );
}

#[test]
fn incremental_missing_edit_chain_uses_full_document_replacement() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let _buffer = open_buffer(&manager, &path, "hello");
    manager
        .sync_buffer_with_edits(&path, "replaced", 99, None, None)
        .expect("missing chain fallback");

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    assert_eq!(
        changes[0].get("text").and_then(Value::as_str),
        Some("replaced")
    );
    assert_eq!(
        changes[0].get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 0 },
            "end": { "line": 0, "character": 5 },
        }))
    );
}

#[test]
fn full_sync_sends_null_range_and_full_text_even_with_edits() {
    let path = PathBuf::from("src").join("main.rs");
    let mut registry = LanguageServerRegistry::new();
    register_test_rust_analyzer(&mut registry);
    let manager = LspClientManager::new(registry);
    let session = test_session_handle("rust-analyzer", &path, BTreeMap::new());
    attach_session(&manager, &session);

    let mut buffer = open_buffer(&manager, &path, "hello");
    let open_revision = buffer.revision();
    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    sync_buffer_edits(&manager, &path, &buffer, open_revision, None);

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    assert!(changes[0].get("range").is_none() || changes[0].get("range") == Some(&Value::Null));
    assert_eq!(
        changes[0].get("text").and_then(Value::as_str),
        Some("hello!")
    );
}

#[test]
fn did_open_still_sends_full_text() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let _buffer = open_buffer(&manager, &path, "fn main() {}");
    assert_eq!(
        last_did_open_text(&session).as_deref(),
        Some("fn main() {}")
    );
    assert!(
        last_notification_params(&session, "textDocument/didChange").is_none(),
        "didOpen must not send didChange"
    );
}

#[test]
fn two_sessions_receive_the_same_incremental_content_changes() {
    let path = PathBuf::from("src").join("main.rs");
    let mut registry = LanguageServerRegistry::new();
    register_test_rust_analyzer(&mut registry);
    registry
        .register(crate::LanguageServerSpec::new(
            "biome",
            "rust",
            ["rs"],
            "dummy-lsp",
            std::iter::empty::<&str>(),
        ))
        .expect("register biome");
    let manager = LspClientManager::new(registry);
    let rust_session = incremental_test_session("rust-analyzer", &path);
    let biome_session = incremental_test_session("biome", &path);
    attach_session(&manager, &rust_session);
    attach_session(&manager, &biome_session);

    let mut buffer = open_buffer(&manager, &path, "hello");
    let open_revision = buffer.revision();
    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    sync_buffer_edits(&manager, &path, &buffer, open_revision, None);

    let rust_changes = last_did_change_content_changes(&rust_session);
    let biome_changes = last_did_change_content_changes(&biome_session);
    assert_eq!(rust_changes, biome_changes);
    assert_eq!(
        rust_changes[0].get("text").and_then(Value::as_str),
        Some("!")
    );
}

#[test]
fn incremental_send_error_recovers_with_full_document() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let mut buffer = open_buffer(&manager, &path, "hello");
    let open_revision = buffer.revision();
    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    session.fail_next_send();
    let failed = manager.sync_buffer_with_edits(
        &path,
        buffer.text(),
        buffer.revision(),
        None,
        buffer.edits_since(open_revision).as_deref(),
    );
    assert!(failed.is_err(), "injected send failure must surface");

    let failed_revision = buffer.revision();
    buffer.insert_text("?");
    manager
        .sync_buffer_with_edits(
            &path,
            buffer.text(),
            buffer.revision(),
            None,
            buffer.edits_since(failed_revision).as_deref(),
        )
        .expect("recovery sync");

    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    assert_eq!(
        changes[0].get("text").and_then(Value::as_str),
        Some("hello!?"),
        "recovery must send the full document, not a stale incremental slice"
    );
    assert_eq!(
        changes[0].get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 0 },
            "end": { "line": 0, "character": 5 },
        }))
    );
}

#[test]
fn close_then_open_then_incremental_edits_work_again() {
    let path = PathBuf::from("src").join("main.rs");
    let (manager, session) = manager_with_incremental_session(&path);
    let mut buffer = open_buffer(&manager, &path, "hello");
    manager.close_buffer(&path).expect("didClose");

    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    manager
        .sync_buffer(&path, buffer.text(), buffer.revision(), None)
        .expect("didOpen after close");
    assert_eq!(last_did_open_text(&session).as_deref(), Some("hello!"));

    let open_revision = buffer.revision();
    buffer.insert_text("?");
    sync_buffer_edits(&manager, &path, &buffer, open_revision, None);
    let changes = last_did_change_content_changes(&session);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].get("text").and_then(Value::as_str), Some("?"));
    assert_eq!(
        changes[0].get("range"),
        Some(&json!({
            "start": { "line": 0, "character": 6 },
            "end": { "line": 0, "character": 6 },
        }))
    );
}

#[test]
fn full_sync_uses_null_range_change() {
    let change = text_document_content_change(TextDocumentSyncKind::FULL, "line one", "updated");

    assert_eq!(change.range, None);
    assert_eq!(change.range_length, None);
    assert_eq!(change.text, "updated");
}

#[cfg(windows)]
#[test]
fn windows_launch_program_candidates_include_command_shims() {
    let candidates = windows_launch_program_candidates("vscode-json-language-server");
    assert!(candidates.contains(&"vscode-json-language-server.cmd".to_owned()));
}

#[cfg(windows)]
#[test]
fn windows_parse_cmd_environment_extracts_variables() {
    let env = parse_windows_cmd_environment(
        "SET PATH=C:\\fnm;C:\\tools\r\nSET FNM_DIR=C:\\Users\\sam\\AppData\\Roaming\\fnm\r\n",
    )
    .expect("fnm env should parse");
    assert_eq!(
        env,
        vec![
            ("PATH".to_owned(), "C:\\fnm;C:\\tools".to_owned()),
            (
                "FNM_DIR".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\fnm".to_owned()
            ),
        ]
    );
}

#[cfg(windows)]
#[test]
fn windows_fnm_environment_keeps_fnm_path_ahead_of_explicit_path() {
    let command = build_lsp_command(
        "node",
        &["--version".to_owned()],
        None,
        &[
            ("PATH".to_owned(), "C:\\custom".to_owned()),
            ("NODE_OPTIONS".to_owned(), "--trace-warnings".to_owned()),
        ],
        Some(&[
            ("PATH".to_owned(), "C:\\fnm".to_owned()),
            (
                "FNM_DIR".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\fnm".to_owned(),
            ),
        ]),
    );
    let vars = command
        .get_envs()
        .filter_map(|(key, value)| {
            Some((
                key.to_string_lossy().into_owned(),
                value?.to_string_lossy().into_owned(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let path = vars.get("PATH").map(String::as_str);
    assert!(
        path.is_some_and(
            |value| value == "C:\\fnm;C:\\custom" || value.starts_with("C:\\fnm;C:\\custom;")
        ),
        "PATH should keep fnm ahead of explicit PATH, got {path:?}"
    );
    assert_eq!(
        vars.get("FNM_DIR").map(String::as_str),
        Some("C:\\Users\\sam\\AppData\\Roaming\\fnm")
    );
    assert_eq!(
        vars.get("NODE_OPTIONS").map(String::as_str),
        Some("--trace-warnings")
    );
}

#[cfg(windows)]
#[test]
fn windows_nvm_environment_keeps_nvm_path_ahead_of_explicit_path() {
    let command = build_lsp_command(
        "node",
        &["--version".to_owned()],
        None,
        &[
            ("PATH".to_owned(), "C:\\custom".to_owned()),
            ("NODE_OPTIONS".to_owned(), "--trace-warnings".to_owned()),
        ],
        Some(&[
            (
                "PATH".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\nvm\\v22.1.0".to_owned(),
            ),
            (
                "NVM_HOME".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\nvm".to_owned(),
            ),
        ]),
    );
    let vars = command
        .get_envs()
        .filter_map(|(key, value)| {
            Some((
                key.to_string_lossy().into_owned(),
                value?.to_string_lossy().into_owned(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let path = vars.get("PATH").map(String::as_str);
    assert!(
        path.is_some_and(|value| value
            == "C:\\Users\\sam\\AppData\\Roaming\\nvm\\v22.1.0;C:\\custom"
            || value.starts_with("C:\\Users\\sam\\AppData\\Roaming\\nvm\\v22.1.0;C:\\custom;")),
        "PATH should keep nvm ahead of explicit PATH, got {path:?}"
    );
    assert_eq!(
        vars.get("NVM_HOME").map(String::as_str),
        Some("C:\\Users\\sam\\AppData\\Roaming\\nvm")
    );
    assert_eq!(
        vars.get("NODE_OPTIONS").map(String::as_str),
        Some("--trace-warnings")
    );
}

#[cfg(windows)]
#[test]
fn windows_should_retry_invalid_exe_format() {
    let error = std::io::Error::from_raw_os_error(193);
    assert!(windows_should_retry_spawn_error(&error));
}

#[cfg(windows)]
#[test]
fn windows_fnm_launch_program_candidates_resolve_absolute_command_shims() {
    let temp_dir = temp_dir();
    std::fs::create_dir_all(&temp_dir).expect("temp dir");
    let candidate_path = temp_dir.join("vscode-json-language-server.cmd");
    std::fs::write(&candidate_path, "@echo off\r\n").expect("candidate");

    let candidates = windows_fnm_launch_program_candidates(
        "vscode-json-language-server",
        &[("PATH".to_owned(), temp_dir.to_string_lossy().into_owned())],
    );
    assert!(candidates.contains(&candidate_path.to_string_lossy().into_owned()));

    let _ = std::fs::remove_file(candidate_path);
    let _ = std::fs::remove_dir(temp_dir);
}

#[cfg(windows)]
#[test]
fn windows_fnm_launch_program_candidates_prefer_windows_shims_over_extensionless_scripts() {
    let temp_dir = temp_dir();
    std::fs::create_dir_all(&temp_dir).expect("temp dir");
    let script_path = temp_dir.join("typescript-language-server");
    let shim_path = temp_dir.join("typescript-language-server.cmd");
    std::fs::write(&script_path, "#!/bin/sh\n").expect("script");
    std::fs::write(&shim_path, "@echo off\r\n").expect("shim");

    let candidates = windows_fnm_launch_program_candidates(
        "typescript-language-server",
        &[("PATH".to_owned(), temp_dir.to_string_lossy().into_owned())],
    );
    assert_eq!(
        candidates.first().map(String::as_str),
        Some(shim_path.to_string_lossy().as_ref())
    );
    assert!(candidates.contains(&script_path.to_string_lossy().into_owned()));

    let _ = std::fs::remove_file(script_path);
    let _ = std::fs::remove_file(shim_path);
    let _ = std::fs::remove_dir(temp_dir);
}

#[cfg(windows)]
#[test]
fn parse_windows_nvm_current_version_extracts_active_version() {
    let version =
        parse_windows_nvm_current_version("v22.1.0\r\n").expect("nvm current should parse");
    assert_eq!(version, "v22.1.0");
}

#[cfg(windows)]
#[test]
fn windows_nvm_node_dir_accepts_version_with_or_without_v_prefix() {
    let temp_dir = temp_dir();
    let version_dir = temp_dir.join("v22.1.0");
    std::fs::create_dir_all(&version_dir).expect("version dir");
    std::fs::write(version_dir.join("node.exe"), []).expect("node exe");

    let resolved = windows_nvm_node_dir(&temp_dir, "22.1.0").expect("node dir");
    assert_eq!(resolved, version_dir);

    let _ = std::fs::remove_file(version_dir.join("node.exe"));
    let _ = std::fs::remove_dir(version_dir);
    let _ = std::fs::remove_dir(temp_dir);
}

#[test]
fn show_message_notifications_map_severity_levels() {
    let params = json!({
        "type": 1,
        "message": "failed to load workspace"
    });
    let notification =
        parse_show_message_notification("rust-analyzer", None, &params).expect("notification");
    assert_eq!(notification.level(), LspNotificationLevel::Error);
    assert_eq!(notification.body_lines(), ["failed to load workspace"]);
    assert!(!notification.active());
}

#[test]
fn show_message_notifications_ignore_non_error_levels() {
    assert!(
        parse_show_message_notification(
            "rust-analyzer",
            None,
            &json!({
                "type": 2,
                "message": "workspace indexing warning"
            }),
        )
        .is_none()
    );
    assert!(
        parse_show_message_notification(
            "rust-analyzer",
            None,
            &json!({
                "type": 3,
                "message": "background info"
            }),
        )
        .is_none()
    );
}

#[test]
fn log_message_error_from_ols_does_not_become_ui_notification() {
    let params = json!({
        "type": 1,
        "message": "Starting Odin Language Server dev-2026-05"
    });
    let root = Path::new(r"P:\odinpong");
    assert!(
        parse_window_message_notification("window/logMessage", "ols", Some(root), &params)
            .is_none()
    );
    let toast = parse_window_message_notification("window/showMessage", "ols", Some(root), &params)
        .expect("showMessage Error still becomes a toast");
    assert_eq!(toast.level(), LspNotificationLevel::Error);
    assert_eq!(
        toast.body_lines(),
        ["Starting Odin Language Server dev-2026-05", r"P:\odinpong"]
    );
}

#[test]
fn session_in_scope_when_open_buffer_is_tracked() {
    let workspace = sample_root("volt");
    let nested = workspace.join("crates").join("editor-lsp");
    let tracked = [workspace.join("src").join("main.rs")];
    assert!(language_server_session_in_workspace_scope(
        Some(workspace.as_path()),
        &tracked,
        &tracked,
        Some(nested.as_path()),
    ));
}

#[test]
fn session_in_scope_when_root_under_project_workspace_without_open_buffer() {
    let workspace = sample_root("volt");
    let session_root = workspace.join("crates").join("editor-lsp");
    let tracked = [workspace.join("src").join("lib.rs")];
    let open: [PathBuf; 0] = [];
    assert!(language_server_session_in_workspace_scope(
        Some(session_root.as_path()),
        &tracked,
        &open,
        Some(workspace.as_path()),
    ));
}

#[test]
fn session_out_of_scope_when_parent_root_and_no_open_buffer() {
    let workspace = sample_root("volt");
    let nested = workspace.join("crates").join("editor-lsp");
    let tracked = [workspace.join("src").join("main.rs")];
    let open: [PathBuf; 0] = [];
    assert!(!language_server_session_in_workspace_scope(
        Some(workspace.as_path()),
        &tracked,
        &open,
        Some(nested.as_path()),
    ));
}

#[test]
fn default_workspace_lists_only_sessions_serving_open_buffers() {
    let root = sample_root("scratch");
    let tracked = [root.join("main.rs")];
    assert!(language_server_session_in_workspace_scope(
        Some(root.as_path()),
        &tracked,
        &tracked,
        None,
    ));
    assert!(!language_server_session_in_workspace_scope(
        Some(root.as_path()),
        &tracked,
        &[],
        None,
    ));
}

#[test]
fn live_session_picker_label_includes_server_and_root() {
    let root = {
        #[cfg(windows)]
        {
            PathBuf::from(r"p:\volt")
        }
        #[cfg(not(windows))]
        {
            PathBuf::from("/volt")
        }
    };
    let with_root = LspLiveSession::new("rust-analyzer", Some(root.clone()));
    assert_eq!(
        with_root.picker_label(),
        format!("rust-analyzer — {}", root.display())
    );
    let no_root = LspLiveSession::new("marksman", None);
    assert_eq!(no_root.picker_label(), "marksman — (no root)");
}

#[test]
fn live_sessions_for_workspace_includes_root_scoped_and_buffer_served() {
    let workspace = sample_root("volt");
    let nested_open = workspace.join("crates").join("editor-lsp").join("lib.rs");
    let under_root_path = workspace.join("src").join("main.rs");
    let nested_root = workspace.join("crates").join("editor-lsp");
    let manager = LspClientManager::new(LanguageServerRegistry::new());

    let parent_session = test_session_handle_in_workspace(
        "rust-analyzer",
        &nested_open,
        Some(workspace.as_path()),
        BTreeMap::new(),
    );
    let under_root_session = test_session_handle_in_workspace(
        "marksman",
        &under_root_path,
        Some(nested_root.as_path()),
        BTreeMap::new(),
    );
    let parent_key = parent_session.key.clone();
    let under_key = under_root_session.key.clone();
    {
        let mut state = manager.state.lock().expect("state lock");
        state.sessions.insert(parent_key.clone(), parent_session);
        state.sessions.insert(under_key.clone(), under_root_session);

        let mut tracked_nested = TrackedBufferState {
            revision: 1,
            version: 1,
            ..TrackedBufferState::default()
        };
        tracked_nested.sessions.insert(parent_key);
        state
            .tracked_buffers
            .insert(nested_open.clone(), tracked_nested);

        let mut tracked_under = TrackedBufferState {
            revision: 1,
            version: 1,
            ..TrackedBufferState::default()
        };
        tracked_under.sessions.insert(under_key);
        state.tracked_buffers.insert(under_root_path, tracked_under);
    }

    // Nested Project Workspace: parent Session included via open buffer;
    // marksman included via root under workspace.
    let listed = manager
        .live_sessions_for_workspace(&[nested_open], Some(workspace.as_path()))
        .expect("list");
    let ids = listed
        .iter()
        .map(|session| session.server_id().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["marksman".to_owned(), "rust-analyzer".to_owned()]);
}

#[test]
fn stop_session_removes_live_session_and_returns_tracked_paths() {
    let root = PathBuf::from(r"P:\workspace");
    let path = root.join("src").join("a.rs");
    let manager = LspClientManager::new(LanguageServerRegistry::new());
    let session = test_session_handle_in_workspace(
        "rust-analyzer",
        &path,
        Some(root.as_path()),
        BTreeMap::new(),
    );
    let session_key = session.key.clone();
    {
        let mut state = manager.state.lock().expect("state lock");
        state.sessions.insert(session_key.clone(), session);
        let mut tracked = TrackedBufferState {
            revision: 1,
            version: 1,
            ..TrackedBufferState::default()
        };
        tracked.sessions.insert(session_key);
        state.tracked_buffers.insert(path.clone(), tracked);
    }

    let stopped_paths = manager
        .stop_session("rust-analyzer", Some(root.as_path()))
        .expect("stop");
    assert_eq!(stopped_paths, vec![path]);
    assert!(
        manager
            .state
            .lock()
            .expect("state lock")
            .sessions
            .is_empty()
    );
}

#[test]
fn sync_buffer_onto_session_attaches_to_exact_root() {
    let root = PathBuf::from(r"P:\workspace");
    let path = root.join("src").join("a.rs");
    let manager = LspClientManager::new(LanguageServerRegistry::new());
    let session = test_session_handle_in_workspace(
        "rust-analyzer",
        &path,
        Some(root.as_path()),
        BTreeMap::new(),
    );
    let session_key = session.key.clone();
    {
        let mut state = manager.state.lock().expect("state lock");
        state.sessions.insert(session_key.clone(), session);
    }

    manager
        .sync_buffer_onto_session(&path, "fn a() {}", 1, "rust-analyzer", Some(root.as_path()))
        .expect("sync onto session");

    let state = manager.state.lock().expect("state lock");
    let tracked = state.tracked_buffers.get(&path).expect("buffer tracked");
    assert!(tracked.sessions.contains(&session_key));
    assert_eq!(state.sessions.len(), 1);
}
