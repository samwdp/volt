const HOOK_MOVE_LEFT: &str = "editor.cursor.move-left";
const HOOK_MOVE_DOWN: &str = "editor.cursor.move-down";
const HOOK_MOVE_UP: &str = "editor.cursor.move-up";
const HOOK_MOVE_RIGHT: &str = "editor.cursor.move-right";
const HOOK_MOVE_WORD_FORWARD: &str = "editor.cursor.move-word-forward";
const HOOK_MOVE_WORD_BACKWARD: &str = "editor.cursor.move-word-backward";
const HOOK_MOVE_WORD_END: &str = "editor.cursor.move-word-end";
const HOOK_MOVE_BIG_WORD_FORWARD: &str = "editor.cursor.move-big-word-forward";
const HOOK_MOVE_BIG_WORD_BACKWARD: &str = "editor.cursor.move-big-word-backward";
const HOOK_MOVE_BIG_WORD_END: &str = "editor.cursor.move-big-word-end";
const HOOK_MOVE_SENTENCE_FORWARD: &str = "editor.cursor.move-sentence-forward";
const HOOK_MOVE_SENTENCE_BACKWARD: &str = "editor.cursor.move-sentence-backward";
const HOOK_MOVE_PARAGRAPH_FORWARD: &str = "editor.cursor.move-paragraph-forward";
const HOOK_MOVE_PARAGRAPH_BACKWARD: &str = "editor.cursor.move-paragraph-backward";
const HOOK_MATCH_PAIR: &str = "editor.cursor.match-pair";
const HOOK_MOVE_LINE_START: &str = "editor.cursor.move-line-start";
const HOOK_MOVE_LINE_FIRST_NON_BLANK: &str = "editor.cursor.move-line-first-non-blank";
const HOOK_MOVE_LINE_END: &str = "editor.cursor.move-line-end";
const HOOK_MOVE_SCREEN_TOP: &str = "editor.cursor.move-screen-top";
const HOOK_MOVE_SCREEN_MIDDLE: &str = "editor.cursor.move-screen-middle";
const HOOK_MOVE_SCREEN_BOTTOM: &str = "editor.cursor.move-screen-bottom";
const HOOK_GOTO_FIRST_LINE: &str = "editor.cursor.goto-first-line";
const HOOK_GOTO_LAST_LINE: &str = "editor.cursor.goto-last-line";
const HOOK_SCROLL_HALF_PAGE_DOWN: &str = "editor.vim.scroll-half-page-down";
const HOOK_SCROLL_HALF_PAGE_UP: &str = "editor.vim.scroll-half-page-up";
const HOOK_SCROLL_PAGE_DOWN: &str = "editor.vim.scroll-page-down";
const HOOK_SCROLL_PAGE_UP: &str = "editor.vim.scroll-page-up";
const HOOK_SCROLL_LINE_DOWN: &str = "editor.vim.scroll-line-down";
const HOOK_SCROLL_LINE_UP: &str = "editor.vim.scroll-line-up";
const HOOK_CURRENT_LINE_TOP: &str = "editor.vim.current-line-top";
const HOOK_CENTER_CURRENT_LINE: &str = "editor.vim.center-current-line";
const HOOK_CURRENT_LINE_BOTTOM: &str = "editor.vim.current-line-bottom";
const HOOK_MODE_INSERT: &str = "editor.mode.insert";
const HOOK_MODE_NORMAL: &str = "editor.mode.normal";
const HOOK_VIM_EDIT: &str = "editor.vim.edit";
const HOOK_VIM_COMMAND_LINE: &str = "editor.vim.command-line";
const HOOK_BUFFER_SAVE: &str = "buffer.save";
const HOOK_BUFFER_CLOSE: &str = "buffer.close";
const HOOK_BUFFER_TOGGLE_LINE_WRAP: &str = "buffer.toggle_line_wrap";
const HOOK_WORKSPACE_SAVE: &str = "workspace.save";
const HOOK_WORKSPACE_NEXT: &str = "workspace.next";
const HOOK_WORKSPACE_PREVIOUS: &str = "workspace.previous";
const HOOK_WORKSPACE_MARK: &str = "workspace.mark";
const HOOK_WORKSPACE_UNMARK: &str = "workspace.unmark";
const HOOK_WORKSPACE_MARKS: &str = "workspace.marks";
const HOOK_WORKSPACE_MARKED_1: &str = "workspace.marked-1";
const HOOK_WORKSPACE_MARKED_2: &str = "workspace.marked-2";
const HOOK_WORKSPACE_MARKED_3: &str = "workspace.marked-3";
const HOOK_WORKSPACE_MARKED_4: &str = "workspace.marked-4";
const HOOK_WORKSPACE_WORKTREE_REMOVE: &str = "workspace.worktree-remove";
const HOOK_WORKSPACE_FORMAT: &str = "workspace.format";
const HOOK_WORKSPACE_FORMATTER_REGISTER: &str = "workspace.formatter.register";
const HOOK_PICKER_OPEN: &str = "ui.picker.open";
const HOOK_PICKER_NEXT: &str = "ui.picker.next";
const HOOK_PICKER_PREVIOUS: &str = "ui.picker.previous";
const HOOK_PICKER_SUBMIT: &str = "ui.picker.submit";
const HOOK_PICKER_CANCEL: &str = "ui.picker.cancel";
const HOOK_QUICKFIX_OPEN: &str = "ui.quickfix.open";
const HOOK_QUICKFIX_NEXT: &str = "ui.quickfix.next";
const HOOK_QUICKFIX_PREVIOUS: &str = "ui.quickfix.previous";
const HOOK_QUICKFIX_TOGGLE_MARK: &str = "ui.quickfix.toggle-mark";
const HOOK_QUICKFIX_CLEAR_MARKS: &str = "ui.quickfix.clear-marks";
const HOOK_QUICKFIX_MARK_ALL: &str = "ui.quickfix.mark-all";
const HOOK_AUTOCOMPLETE_TRIGGER: &str = autocomplete_hooks::TRIGGER;
const HOOK_AUTOCOMPLETE_NEXT: &str = autocomplete_hooks::NEXT;
const HOOK_AUTOCOMPLETE_PREVIOUS: &str = autocomplete_hooks::PREVIOUS;
const HOOK_AUTOCOMPLETE_ACCEPT: &str = autocomplete_hooks::ACCEPT;
const HOOK_AUTOCOMPLETE_CANCEL: &str = autocomplete_hooks::CANCEL;
const HOOK_HOVER_TOGGLE: &str = hover_hooks::TOGGLE;
const HOOK_HOVER_FOCUS: &str = hover_hooks::FOCUS;
const HOOK_HOVER_NEXT: &str = hover_hooks::NEXT;
const HOOK_HOVER_PREVIOUS: &str = hover_hooks::PREVIOUS;
const HOOK_POPUP_TOGGLE: &str = "ui.popup.toggle";
const HOOK_POPUP_NEXT: &str = "ui.popup.next";
const HOOK_POPUP_PREVIOUS: &str = "ui.popup.previous";
const HOOK_WORKSPACE_DOCK_TOGGLE: &str = "ui.workspace-dock.toggle";
const HOOK_WORKSPACE_DOCK_PREVIOUS: &str = "ui.workspace-dock.previous";
const HOOK_WORKSPACE_DOCK_NEXT: &str = "ui.workspace-dock.next";
const HOOK_ACP_DOCK_TOGGLE: &str = "ui.acp-dock.toggle";
const HOOK_ACP_DOCK_PREVIOUS: &str = "ui.acp-dock.previous";
const HOOK_ACP_DOCK_NEXT: &str = "ui.acp-dock.next";
const HOOK_ACP_DISCONNECT: &str = "ui.acp.disconnect";
const HOOK_ACP_PERMISSION_APPROVE: &str = "ui.acp.permission-approve";
const HOOK_ACP_PERMISSION_DENY: &str = "ui.acp.permission-deny";
const HOOK_ACP_PICK_SESSION: &str = "ui.acp.pick-session";
const HOOK_ACP_NEW_SESSION: &str = "ui.acp.new-session";
const HOOK_ACP_PICK_MODE: &str = "ui.acp.pick-mode";
const HOOK_ACP_PICK_MODEL: &str = "ui.acp.pick-model";
const HOOK_ACP_CYCLE_MODE: &str = "ui.acp.cycle-mode";
const HOOK_ACP_SWITCH_PANE: &str = "ui.acp.switch-pane";
const HOOK_ACP_COMPLETE_SLASH: &str = "ui.acp.complete-slash";
const HOOK_ACP_FOCUS_INPUT: &str = "ui.acp.focus-input";
const HOOK_PANE_SPLIT_HORIZONTAL: &str = "ui.pane.split-horizontal";
const HOOK_PANE_SPLIT_VERTICAL: &str = "ui.pane.split-vertical";
const HOOK_PANE_CLOSE: &str = "ui.pane.close";
const HOOK_PANE_SWITCH_SPLIT: &str = "ui.pane.switch-split";
const HOOK_WORKSPACE_WINDOW_LEFT: &str = "ui.workspace.window-left";
const HOOK_WORKSPACE_WINDOW_DOWN: &str = "ui.workspace.window-down";
const HOOK_WORKSPACE_WINDOW_UP: &str = "ui.workspace.window-up";
const HOOK_WORKSPACE_WINDOW_RIGHT: &str = "ui.workspace.window-right";
const INTERACTIVE_READONLY_KIND: &str = "interactive-readonly";
const INTERACTIVE_INPUT_KIND: &str = "interactive-input";
const ACP_BUFFER_KIND: &str = buffer_kinds::ACP;
const BROWSER_KIND: &str = buffer_kinds::BROWSER;
const PDF_BUFFER_KIND: &str = buffer_kinds::PDF;
const SQLS_SERVER_ID: &str = "sqls";
const DB_CONNECT_KIND: &str = buffer_kinds::DB_CONNECT;
const DB_QUERY_KIND: &str = buffer_kinds::DB_QUERY;
const DB_CONNECTIONS_KIND: &str = buffer_kinds::DB_CONNECTIONS;
const DB_SCHEMA_KIND: &str = buffer_kinds::DB_SCHEMA;
const DB_HISTORY_KIND: &str = buffer_kinds::DB_HISTORY;
const DB_SNIPPETS_KIND: &str = buffer_kinds::DB_SNIPPETS;
const DB_RESULTS_KIND: &str = buffer_kinds::DB_RESULTS;
const DB_DASHBOARD_KIND: &str = buffer_kinds::DB_DASHBOARD;
const DB_SIDEBAR_KIND: &str = buffer_kinds::DB_SIDEBAR;
const DB_CONNECT_BUFFER_NAME: &str = "*db-connect*";
const DB_CONNECTIONS_BUFFER_NAME: &str = "*db-connections*";
const DB_SCHEMA_BUFFER_NAME: &str = "*db-schema*";
const DB_HISTORY_BUFFER_NAME: &str = "*db-history*";
const DB_SNIPPETS_BUFFER_NAME: &str = "*db-snippets*";
const DB_RESULTS_BUFFER_NAME: &str = "*db-results*";
const DB_DASHBOARD_BUFFER_NAME: &str = "*db-dashboard*";
const DB_SIDEBAR_BUFFER_NAME: &str = "*db-sidebar*";
const DB_EDITOR_SECTION: &str = "Editor";
const DB_CONNECTIONS_SECTION: &str = "Connections";
const DB_TABLES_SECTION: &str = "Tables";
const DB_OUTPUT_SECTION: &str = "Output";
const DB_MULTIVIEW_LEFT_WEIGHT: u32 = 1;
const DB_MULTIVIEW_RIGHT_WEIGHT: u32 = 3;
const DAP_BREAKPOINTS_KIND: &str = buffer_kinds::DAP_BREAKPOINTS;
const DAP_LOCALS_KIND: &str = buffer_kinds::DAP_LOCALS;
const DAP_LOCALS_BUFFER_NAME: &str = "*dap-locals*";
const DAP_REPL_BUFFER_NAME: &str = "*dap-repl*";
const DAP_EVAL_BUFFER_NAME: &str = "*dap-eval*";
const DAP_LOCALS_SECTION: &str = "Locals";
const DAP_EXPRESSIONS_SECTION: &str = "Expressions";
const DEBUG_LAYOUT_BREAKPOINTS_WEIGHT: u32 = 1;
const DEBUG_LAYOUT_EDITOR_WEIGHT: u32 = 3;
const DEBUG_LAYOUT_LOCALS_WEIGHT: u32 = 2;
const HOOK_BROWSER_OPEN: &str = browser_hooks::OPEN;
const HOOK_BROWSER_OPEN_BUFFER: &str = browser_hooks::OPEN_BUFFER;
const HOOK_BROWSER_OPEN_POPUP: &str = browser_hooks::OPEN_POPUP;
const HOOK_BROWSER_URL: &str = browser_hooks::URL;
const HOOK_BROWSER_FOCUS_INPUT: &str = browser_hooks::FOCUS_INPUT;
const HOOK_BROWSER_SUBMIT: &str = browser_hooks::SUBMIT;
const HOOK_TERMINAL_OPEN_POPUP: &str = terminal_hooks::OPEN_POPUP;
const HOOK_IMAGE_ZOOM_IN: &str = image_hooks::ZOOM_IN;
const HOOK_IMAGE_ZOOM_OUT: &str = image_hooks::ZOOM_OUT;
const HOOK_IMAGE_ZOOM_RESET: &str = image_hooks::ZOOM_RESET;
const HOOK_IMAGE_TOGGLE_MODE: &str = image_hooks::TOGGLE_MODE;
const HOOK_MARKDOWN_PRETTY_TOGGLE: &str = "markdown.pretty.toggle";
const HOOK_RAINBOW_PARENS_TOGGLE: &str = "rainbow.parens.toggle";
const HOOK_SHOW_PAREN_TOGGLE: &str = "show-paren.toggle";
const HOOK_PDF_NEXT_PAGE: &str = pdf_hooks::NEXT_PAGE;
const HOOK_PDF_PREVIOUS_PAGE: &str = pdf_hooks::PREVIOUS_PAGE;
const HOOK_PDF_ROTATE_CLOCKWISE: &str = pdf_hooks::ROTATE_CLOCKWISE;
const HOOK_PDF_DELETE_PAGE: &str = pdf_hooks::DELETE_PAGE;
const AUTOCOMPLETE_BUFFER_PROVIDER: &str = "buffer";
const AUTOCOMPLETE_DB_PROVIDER: &str = "db";
const AUTOCOMPLETE_LSP_PROVIDER: &str = "lsp";
const HOVER_PROVIDER_TEST: &str = "test-hover";
const HOVER_PROVIDER_LSP: &str = "lsp";
const HOVER_PROVIDER_SIGNATURE_HELP: &str = "signature-help";
const HOVER_PROVIDER_DIAGNOSTICS: &str = "diagnostics";
const HOOK_LSP_START: &str = lsp_hooks::START;
const HOOK_LSP_STOP: &str = lsp_hooks::STOP;
const HOOK_LSP_RESTART: &str = lsp_hooks::RESTART;
const HOOK_LSP_LOG: &str = lsp_hooks::LOG;
const HOOK_LSP_DEFINITION: &str = lsp_hooks::DEFINITION;
const HOOK_LSP_REFERENCES: &str = lsp_hooks::REFERENCES;
const HOOK_LSP_IMPLEMENTATION: &str = lsp_hooks::IMPLEMENTATION;
const HOOK_LSP_DIAGNOSTICS: &str = lsp_hooks::DIAGNOSTICS;
const HOOK_LSP_CODE_ACTIONS: &str = lsp_hooks::CODE_ACTIONS;
const HOOK_LSP_COPILOT_SIGN_IN: &str = lsp_hooks::COPILOT_SIGN_IN;
const HOOK_LSP_COPILOT_SIGN_OUT: &str = lsp_hooks::COPILOT_SIGN_OUT;
const HOOK_LSP_INSTALL: &str = lsp_hooks::INSTALL;
const HOOK_DAP_START: &str = dap_hooks::START;
const HOOK_DAP_START_LAST: &str = dap_hooks::START_LAST;
const HOOK_DAP_START_RECENT: &str = dap_hooks::START_RECENT;
const HOOK_DAP_STOP: &str = dap_hooks::STOP;
const HOOK_DAP_RESTART: &str = dap_hooks::RESTART;
const HOOK_DAP_CONTINUE: &str = dap_hooks::CONTINUE;
const HOOK_DAP_PAUSE: &str = dap_hooks::PAUSE;
const HOOK_DAP_STEP: &str = dap_hooks::STEP;
const HOOK_DAP_STEP_INTO: &str = dap_hooks::STEP_INTO;
const HOOK_DAP_STEP_OUT: &str = dap_hooks::STEP_OUT;
const HOOK_DAP_LOG: &str = dap_hooks::LOG;
const HOOK_DAP_TOGGLE_BREAKPOINT: &str = dap_hooks::TOGGLE_BREAKPOINT;
const HOOK_DAP_DELETE_BREAKPOINT: &str = dap_hooks::DELETE_BREAKPOINT;
const HOOK_DAP_OPEN_BREAKPOINTS: &str = dap_hooks::OPEN_BREAKPOINTS;
const HOOK_DAP_EXPRESSIONS_ADD: &str = dap_hooks::EXPRESSIONS_ADD;
const HOOK_DAP_EXPRESSIONS_REMOVE: &str = dap_hooks::EXPRESSIONS_REMOVE;
const HOOK_DAP_EVAL: &str = dap_hooks::EVAL;
const HOOK_DAP_EVAL_AT_POINT: &str = dap_hooks::EVAL_AT_POINT;
const HOOK_DAP_REPL: &str = dap_hooks::REPL;
const HOOK_DAP_SWITCH_THREAD: &str = dap_hooks::SWITCH_THREAD;
const HOOK_DAP_SWITCH_STACK_FRAME: &str = dap_hooks::SWITCH_STACK_FRAME;
const HOOK_DAP_BREAKPOINT_CONDITION: &str = dap_hooks::BREAKPOINT_CONDITION;
const HOOK_DAP_BREAKPOINT_HIT_CONDITION: &str = dap_hooks::BREAKPOINT_HIT_CONDITION;
const HOOK_DAP_BREAKPOINT_LOG_MESSAGE: &str = dap_hooks::BREAKPOINT_LOG_MESSAGE;
const HOOK_DAP_TOGGLE_VARIABLE: &str = dap_hooks::TOGGLE_VARIABLE;
const HOOK_DAP_GOTO_BREAKPOINT: &str = dap_hooks::GOTO_BREAKPOINT;
const HOOK_DAP_INSTALL: &str = dap_hooks::INSTALL;
const DAP_SESSIONS_BUFFER_NAME: &str = "*dap-sessions*";
const DAP_LOG_BUFFER_NAME: &str = "*dap-log*";
const DAP_BREAKPOINTS_BUFFER_NAME: &str = "*dap-breakpoints*";
const DEBUG_FRINGE_VERIFIED_GLYPH: &str = "●";
const DEBUG_FRINGE_PENDING_GLYPH: &str = "○";
const DEBUG_FRINGE_UNVERIFIED_GLYPH: &str = "✕";
const DEBUG_FRINGE_EXECUTION_GLYPH: &str = "▶";
const TOKEN_DEBUG_FRINGE_BREAKPOINT: &str = "debug.fringe.breakpoint";
const TOKEN_DEBUG_FRINGE_PENDING: &str = "debug.fringe.pending";
const TOKEN_DEBUG_FRINGE_EXECUTION: &str = "debug.fringe.execution";
const TOKEN_DEBUG_LINE_EXECUTION: &str = "debug.line.execution";
const COPILOT_LANGUAGE_SERVER: &str = "copilot-language-server";
const ACP_INPUT_PLACEHOLDER: &str =
    "Type / for commands, @ for files. Paste images with Ctrl+Shift+V";
const QUICKFIX_BUFFER_NAME: &str = "*quickfix*";
const QUICKFIX_POPUP_TITLE: &str = "Quickfix";
const GIT_STATUS_KIND: &str = buffer_kinds::GIT_STATUS;
const GIT_COMMIT_KIND: &str = buffer_kinds::GIT_COMMIT;
const GIT_DIFF_KIND: &str = buffer_kinds::GIT_DIFF;
const GIT_LOG_KIND: &str = buffer_kinds::GIT_LOG;
const GIT_STASH_KIND: &str = buffer_kinds::GIT_STASH;
const HOOK_PLUGIN_EVALUATE: &str = plugin_hooks::EVALUATE;
const PLUGIN_EVALUATE_SEPARATOR_PREFIX: &str = plugin_hooks::EVALUATE_SEPARATOR_PREFIX;
const HOOK_PLUGIN_RUN_COMMAND: &str = plugin_hooks::RUN_COMMAND;
const HOOK_PLUGIN_RERUN_COMMAND: &str = plugin_hooks::RERUN_COMMAND;
const HOOK_PLUGIN_RELOAD_USER_LIBRARY: &str = "plugin.reload-user-library";
const HOOK_PLUGIN_SWITCH_PANE: &str = plugin_hooks::SWITCH_PANE;
const HOOK_TREESITTER_RECOMPILE_INSTALLED: &str = "treesitter.recompile-installed";
const HOOK_GIT_STATUS_OPEN_POPUP: &str = git_hooks::STATUS_OPEN_POPUP;
const HOOK_GIT_DIFF_OPEN: &str = git_hooks::DIFF_OPEN;
const HOOK_GIT_LOG_OPEN: &str = git_hooks::LOG_OPEN;
const HOOK_GIT_STASH_LIST_OPEN: &str = git_hooks::STASH_LIST_OPEN;
const HOOK_OIL_OPEN: &str = oil_hooks::OPEN;
const HOOK_OIL_OPEN_PARENT: &str = oil_hooks::OPEN_PARENT;
const HOOK_OIL_ACTION: &str = oil_hooks::ACTION;
const HOOK_OIL_GIT_WORKTREE: &str = oil_hooks::GIT_WORKTREE;
const HOOK_DB_CONNECT: &str = db_hooks::CONNECT;
const HOOK_DB_DISCONNECT: &str = db_hooks::DISCONNECT;
const HOOK_DB_SHOW_TABLES: &str = db_hooks::SHOW_TABLES;
const HOOK_DB_NEW_QUERY_BUFFER: &str = db_hooks::NEW_QUERY_BUFFER;
const HOOK_DB_EXECUTE_SQL: &str = db_hooks::EXECUTE_SQL;
const HOOK_DB_SHOW_CONNECTIONS: &str = db_hooks::SHOW_CONNECTIONS;
const HOOK_DB_SHOW_HISTORY: &str = db_hooks::SHOW_HISTORY;
const HOOK_DB_SHOW_SNIPPETS: &str = db_hooks::SHOW_SNIPPETS;
const HOOK_DB_SAVE_SNIPPET: &str = db_hooks::SAVE_SNIPPET;
const HOOK_DB_REFRESH_SCHEMA: &str = db_hooks::REFRESH_SCHEMA;
const HOOK_DB_ACTIVATE_LINE: &str = db_hooks::ACTIVATE_LINE;
const HOOK_DB_DASHBOARD: &str = db_hooks::DASHBOARD;
const HOOK_DB_MULTIVIEW: &str = db_hooks::MULTIVIEW;
const GIT_ACTION_STAGE_FILE: &str = git_actions::STAGE_FILE;
const GIT_ACTION_UNSTAGE_FILE: &str = git_actions::UNSTAGE_FILE;
const GIT_ACTION_SHOW_COMMIT: &str = git_actions::SHOW_COMMIT;
const GIT_ACTION_SHOW_STASH: &str = git_actions::SHOW_STASH;
const GIT_SECTION_HEADERS: &str = git_sections::HEADERS;
const GIT_SECTION_IN_PROGRESS: &str = git_sections::IN_PROGRESS;
const GIT_SECTION_STAGED: &str = git_sections::STAGED;
const GIT_SECTION_UNSTAGED: &str = git_sections::UNSTAGED;
const GIT_SECTION_UNTRACKED: &str = git_sections::UNTRACKED;
const GIT_SECTION_STASHES: &str = git_sections::STASHES;
const GIT_SECTION_UNPULLED: &str = git_sections::UNPULLED;
const GIT_SECTION_UNPUSHED: &str = git_sections::UNPUSHED;
const GIT_SECTION_COMMIT: &str = git_sections::COMMIT;
const PDF_ROTATION_FULL_CIRCLE: i64 = 360;
const TOKEN_GIT_STATUS_SECTION_HEADER: &str = "git.status.section.header";
const TOKEN_GIT_STATUS_SECTION_COUNT: &str = "git.status.section.count";
const TOKEN_GIT_STATUS_HEADER_LABEL: &str = "git.status.header.label";
const TOKEN_GIT_STATUS_HEADER_VALUE: &str = "git.status.header.value";
const TOKEN_GIT_STATUS_HEADER_HASH: &str = "git.status.header.hash";
const TOKEN_GIT_STATUS_HEADER_SUMMARY: &str = "git.status.header.summary";
const TOKEN_GIT_STATUS_IN_PROGRESS: &str = "git.status.in-progress";
const TOKEN_GIT_STATUS_ENTRY_ADDED: &str = "git.status.entry.added";
const TOKEN_GIT_STATUS_ENTRY_MODIFIED: &str = "git.status.entry.modified";
const TOKEN_GIT_STATUS_ENTRY_DELETED: &str = "git.status.entry.deleted";
const TOKEN_GIT_STATUS_ENTRY_RENAMED: &str = "git.status.entry.renamed";
const TOKEN_GIT_STATUS_ENTRY_COPIED: &str = "git.status.entry.copied";
const TOKEN_GIT_STATUS_ENTRY_UPDATED: &str = "git.status.entry.updated";
const TOKEN_GIT_STATUS_ENTRY_CHANGED: &str = "git.status.entry.changed";
const TOKEN_GIT_STATUS_ENTRY_UNTRACKED: &str = "git.status.entry.untracked";
const TOKEN_GIT_STATUS_ENTRY_PATH: &str = "git.status.entry.path";
const TOKEN_GIT_STATUS_COMMIT_HASH: &str = "git.status.commit.hash";
const TOKEN_GIT_STATUS_COMMIT_SUMMARY: &str = "git.status.commit.summary";
const TOKEN_GIT_STATUS_STASH_NAME: &str = "git.status.stash.name";
const TOKEN_GIT_STATUS_STASH_SUMMARY: &str = "git.status.stash.summary";
const TOKEN_GIT_STATUS_COMMAND: &str = "git.status.command";
const TOKEN_GIT_STATUS_MESSAGE: &str = "git.status.message";
const TOKEN_COMMANDLINE_BACKGROUND: &str = "ui.commandline.background";
const TOKEN_STATUSLINE_ACTIVE: &str = "ui.statusline.active";
const TOKEN_STATUSLINE_FOREGROUND: &str = "ui.statusline.foreground";
const TOKEN_STATUSLINE_INACTIVE: &str = "ui.statusline.inactive";
const TOKEN_STATUSLINE_INACTIVE_FOREGROUND: &str = "ui.statusline.inactive.foreground";
const TOKEN_PICKER_BACKGROUND: &str = "ui.picker.background";
const TOKEN_PICKER_FOREGROUND: &str = "ui.picker.foreground";
const TOKEN_PICKER_MUTED: &str = "ui.picker.muted";
const TOKEN_PICKER_SUBTLE: &str = "ui.picker.subtle";
const TOKEN_PICKER_BORDER: &str = "ui.picker.border";
const TOKEN_PICKER_SELECTION: &str = "ui.picker.selection";
const TOKEN_DIAGNOSTIC_ERROR: &str = "ui.diagnostic.error";
const TOKEN_DIAGNOSTIC_WARNING: &str = "ui.diagnostic.warning";
const TOKEN_DIAGNOSTIC_INFO: &str = "ui.diagnostic.info";
const TOKEN_LINE_NUMBER: &str = "ui.line-number";
const TOKEN_LINE_NUMBER_CURRENT: &str = "ui.line-number.current";
const TOKEN_CURRENT_LINE: &str = "ui.current-line";
const TOKEN_SHOW_PAREN_MATCH: &str = "ui.show-paren.match";
const TOKEN_SHOW_PAREN_MISMATCH: &str = "ui.show-paren.mismatch";
const TOKEN_PANE_INACTIVE: &str = "ui.pane.inactive";
const TOKEN_GHOST_TEXT: &str = "ui.ghost-text";
const TOKEN_HEADERLINE: &str = "ui.headerline";
const TOKEN_HEADERLINE_BACKGROUND: &str = "ui.headerline.background";
const GIT_FRINGE_BAR_WIDTH: u32 = 3;
const MOUSE_WHEEL_SCROLL_LINES: i32 = 3;
const OIL_BUFFER_NAME: &str = "*oil*";
const OIL_PREVIEW_BUFFER_NAME: &str = "*oil-preview*";
const OIL_HELP_BUFFER_NAME: &str = "*oil-help*";
const LSP_LOG_BUFFER_PREFIX: &str = "*lsp-log ";
const LSP_METADATA_BUFFER_KIND: &str = "lsp-metadata";
const OIL_PREVIEW_KIND: &str = "oil-preview";
const OIL_HELP_KIND: &str = "oil-help";
const HOOK_INPUT_SUBMIT: &str = input_hooks::SUBMIT;
const HOOK_INPUT_CLEAR: &str = input_hooks::CLEAR;
const OPTION_LINE_NUMBER_RELATIVE: &str = "ui.line-number.relative";
const OPTION_FONT: &str = "font";
const OPTION_FONT_SIZE: &str = "font_size";
const OPTION_CURSOR_ROUNDNESS: &str = "cursor_roundness";
const OPTION_CORNER_RADIUS: &str = "corner_radius";
const OPTION_SCROLL_OFF: &str = "scrolloff";
const OPTION_EMOJI_FONT: &str = "emoji_font";
const OPTION_EMOJI_FONT_SIZE: &str = "emoji_font_size";
const SEARCH_PICKER_ITEM_LIMIT: usize = 512;
const GIT_LOG_LIMIT: usize = 10;
const GIT_LOG_VIEW_LIMIT: usize = 200;
const DEFAULT_WORKSPACE_ROOT_SEARCH_DEPTH: usize = 6;
const BUNDLED_ICON_FONT_SEARCH_DEPTH: usize = 6;
const DEFERRED_ICON_FONT_IDLE_DELAY: Duration = Duration::from_millis(1500);
const DEFERRED_EMOJI_FONT_IDLE_DELAY: Duration = Duration::from_secs(5);
const BUNDLED_ICON_FONT_DIR_CANDIDATES: &[&[&str]] =
    &[&["crates", "volt", "assets", "font"], &["assets", "font"]];
const BUNDLED_ICON_FONT_FILES: &[&str] = &[
    "NFM.ttf",
    "all-the-icons.ttf",
    "file-icons.ttf",
    "fontawesome.ttf",
    "material-design-icons.ttf",
    "octicons.ttf",
    "weathericons.ttf",
];
#[cfg(target_os = "windows")]
const SYSTEM_EMOJI_FONT_CANDIDATES: &[&str] = &[
    r"C:\Windows\Fonts\seguiemj.ttf",
    r"C:\Windows\Fonts\segoeui.ttf",
    r"C:\Windows\Fonts\segoeuil.ttf",
    r"C:\Windows\Fonts\seguili.ttf",
    r"C:\Windows\Fonts\eui.ttf",
];
#[cfg(target_os = "macos")]
const SYSTEM_EMOJI_FONT_CANDIDATES: &[&str] =
    &["/System/Library/Fonts/Supplemental/Apple Color Emoji.ttf"];

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
const SYSTEM_EMOJI_FONT_CANDIDATES: &[&str] = &[
    "/usr/share/fonts/truetype/noto/NotoColorEmoji-Regular.ttf",
    "/usr/share/fonts/truetype/noto/Noto-Color-Emoji.ttf",
    "/usr/share/fonts/opentype/noto/NotoColorEmoji-Regular.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];

#[cfg(target_os = "windows")]
const SYSTEM_ICON_FONT_CANDIDATES: &[&str] = &[
    r"C:\Windows\Fonts\seguisym.ttf",
    r"C:\Windows\Fonts\eufont.ttf",
];
#[cfg(target_os = "macos")]
const SYSTEM_ICON_FONT_CANDIDATES: &[&str] = &[
    "/System/Library/Fonts/Supplemental/Apple Symbols.ttf",
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
];
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
const SYSTEM_ICON_FONT_CANDIDATES: &[&str] = &[
    "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
    "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/opentype/noto/NotoSansSymbols2-Regular.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];
const WINDOW_ICON_BYTES: &[u8] = include_bytes!(concat!(
    core::env!("CARGO_MANIFEST_DIR"),
    "/../volt/assets/logo.png"
));
const ERROR_LOG_MAX_ENTRIES: usize = 200;
const ERROR_LOG_FILE_NAME: &str = "errors.log";
const ACTIVE_THEME_STATE_FILE_NAME: &str = "active-theme.txt";
const MARK_LIST_FILE_NAME: &str = "marked-workspaces.txt";
const TYPING_PROFILE_LOG_FILE_NAME: &str = "typing-profile.log";
const TYPING_PROFILE_MAX_FRAMES: usize = 10_000;
const TYPING_PROFILE_SLOW_FRAME_THRESHOLD: Duration = Duration::from_millis(8);
const FRAME_PACING_TARGET_120FPS: Duration = Duration::from_nanos(8_333_333);
const FPS_OVERLAY_HISTORY_FRAMES: usize = 120;
const FRAME_PACING_YIELD_THRESHOLD: Duration = Duration::from_millis(1);
const FRAME_PACING_TYPING_IDLE_THRESHOLD: Duration = Duration::from_millis(150);
const LSP_SYNC_TYPING_IDLE_THRESHOLD: Duration = Duration::from_millis(150);
const TYPING_EVENT_BATCH_LIMIT: usize = 24;
const TYPING_EVENT_BATCH_TIME_BUDGET: Duration = Duration::from_millis(2);
const GIT_SUMMARY_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const GIT_FRINGE_REFRESH_DEBOUNCE: Duration = Duration::from_millis(150);
const GIT_REFRESH_TYPING_IDLE_THRESHOLD: Duration = Duration::from_millis(750);
const SYNTAX_WINDOW_MIN_LINES: usize = 256;
const SYNTAX_WINDOW_MARGIN_LINES: usize = 96;
const NOTIFICATION_AUTO_DISMISS: Duration = Duration::from_secs(5);
const NOTIFICATION_VISIBLE_LIMIT: usize = 3;
const NOTIFICATION_MAX_STORED: usize = 12;
const NOTIFICATION_STACK_GAP: i32 = 10;
const NOTIFICATION_MAX_BODY_LINES: usize = 4;
const IMAGE_ZOOM_STEP: f32 = 1.25;
const IMAGE_ZOOM_MIN: f32 = 0.1;
const IMAGE_ZOOM_MAX: f32 = 8.0;

// ─── Local constants (formerly from user modules) ────────────────────────────
const BROWSER_BUFFER_NAME: &str = "*browser*";
const AUTOCOMPLETE_NEXT_CHORD: &str = "Ctrl+n";
const AUTOCOMPLETE_PREVIOUS_CHORD: &str = "Ctrl+p";
