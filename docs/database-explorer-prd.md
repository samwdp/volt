# Database Explorer PRD

## 1. Executive Summary

- **Problem Statement**: Volt does not provide built-in database access, so users must leave the editor to inspect schema, run SQL, and manage connections. This breaks the editor-native workflow and makes iterative data work slower.
- **Proposed Solution**: Add a built-in SQL database explorer that supports raw connection strings, secure remembered connections, oil-style schema browsing, SQL query buffers, buffer-native execution via `Ctrl+c Ctrl+c`, and database-aware completion/LSP integration.
- **Success Criteria**:
  - Support successful connect, show-tables, and execute-query flows for `SQLite`, `PostgreSQL`, and `SQL Server / Azure SQL`.
  - Persist remembered connections with encrypted secret storage and never write raw connection strings into source, logs, or plain text config.
  - Expose all database commands under `db.*`.
  - Use SQL tree-sitter highlighting in database query buffers.
  - Provide table and column autocomplete from the active connection schema.

## 2. User Experience & Functionality

- **User Personas**:
  - Solo developer using Volt as primary editor and wanting an in-editor database viewer.
  - Backend developer working across local SQLite/Postgres and cloud Azure SQL.
  - Debugger/operator running ad hoc SQL during development or incident response.

- **User Stories**:
  - As a user, I want to connect to a database from a raw connection string so that I can work with both local and cloud databases inside Volt.
  - As a user, I want Volt to remember prior connections securely so that I do not need to re-enter connection details every session.
  - As a user, I want `db.show-tables` to open an oil-style buffer so that I can browse database objects in an editor-native UI.
  - As a user, I want a SQL query buffer with syntax highlighting so that I can write and inspect SQL comfortably.
  - As a user, I want `Ctrl+c Ctrl+c` in a SQL buffer to execute SQL so that query execution fits the rest of the editor workflow.
  - As a user, I want autocomplete and SQL LSP support so that I can discover tables and columns while writing queries.

- **Acceptance Criteria**:
  - `db.connect` accepts a raw connection string at runtime and creates an in-memory active connection session.
  - Volt never hardcodes connection strings in repo code, compiled user packages, tests with real secrets, or plain text config.
  - Remembered connections store display metadata separately from secret material.
  - Secret material is encrypted at rest. Hashes may be used as identifiers/fingerprints but not as the only stored representation.
  - `db.show-tables` opens an oil-style buffer listing tables for the active connection.
  - Database query buffers are associated with a connection id and SQL dialect.
  - `Ctrl+c Ctrl+c` executes selected SQL when a selection exists; otherwise it executes the current statement or active buffer scope defined by the buffer mode.
  - Query results and execution errors render in Volt buffers or popups without crashing the runtime.
  - SQL tree-sitter highlighting is enabled in database query buffers.
  - Completion provider returns tables and columns for the active connection.
  - SQL LSP attaches to SQL query buffers where supported by the selected dialect/backend.

- **Non-Goals**:
  - Support for non-SQL databases in MVP.
  - ORM, migration authoring, or schema migration orchestration.
  - Visual ER diagramming.
  - Cloud-specific auth flows beyond raw connection strings.
  - Cross-device credential sync.

## 3. AI System Requirements (If Applicable)

- **Tool Requirements**:
  - No AI model is required for MVP.
  - Required runtime/tooling components:
    - DB adapter layer for `SQLite`, `PostgreSQL`, and `SQL Server / Azure SQL`
    - SQL tree-sitter grammar
    - SQL LSP integration
    - Secure secret storage
    - Schema introspection service
    - Completion provider

- **Evaluation Strategy**:
  - Run integration tests that validate connect, list-tables, and execute-query against fixture databases for each supported engine.
  - Validate completion results against known schemas for tables and columns.
  - Validate buffer metadata and LSP attachment behavior for DB query buffers.
  - Add negative tests to ensure secrets never appear in logs, panic output, or persisted plain text files.

## 4. Technical Specifications

- **Architecture Overview**:
  - Add a dedicated runtime subsystem, likely a new crate such as `editor-db`, to own:
    - connection session lifecycle
    - secure remembered connection metadata
    - schema introspection
    - execution requests/results
    - schema cache for autocomplete
  - Expose database functionality through user-facing commands:
    - `db.connect`
    - `db.disconnect`
    - `db.show-tables`
    - `db.new-query-buffer`
    - `db.execute-sql`
    - `db.show-connections`
  - Represent DB query buffers as normal Volt buffers with attached metadata:
    - connection id
    - engine kind
    - database/schema context
    - execution mode / dialect
  - Route `Ctrl+c Ctrl+c` in DB query buffers to `db.execute-sql`.
  - Render schema browsing via an oil-style buffer backed by introspection output.
  - Build completion provider on top of active connection schema cache.
  - Attach SQL LSP to DB query buffers using dialect-aware configuration.

- **Integration Points**:
  - `editor-core`
    - runtime service registration
    - command registration
    - hook/keymap integration
    - buffer-local metadata plumbing
  - `editor-sdl`
    - oil-style schema buffer rendering
    - result buffer and error surface presentation
    - `Ctrl+c Ctrl+c` execution chord handling
  - `editor-syntax`
    - SQL tree-sitter registration for DB buffers
  - `editor-lsp`
    - SQL LSP attachment and configuration
  - `user`
    - declarative `db.*` command exposure
    - default keybindings
  - `editor-jobs` or equivalent async execution path
    - background query execution
    - cancellation and long-running query support

- **Security & Privacy**:
  - Raw connection strings must never be stored in repo code, user package source, logs, or plain text config files.
  - Remembered connections must encrypt secrets at rest.
  - Preferred design is OS-backed secret storage:
    - Windows Credential Manager
    - macOS Keychain
    - Linux Secret Service / keyring
  - Persist non-secret metadata separately in a config file:
    - user-defined alias
    - engine kind
    - host/server display label
    - database name
    - last-used timestamp
    - secret reference id
  - If keyring support is unavailable on a target OS, fallback behavior should be explicit:
    - MVP: refuse persistence and keep session-only secret in memory
    - v1.1+: evaluate encrypted local file fallback if truly needed
  - Redact secrets from command output, panic logs, telemetry, crash reports, and displayed buffers.

## 5. Risks & Roadmap

- **Phased Rollout**:
  - **MVP**
    - `db.connect`, `db.disconnect`, `db.show-tables`, `db.new-query-buffer`, `db.execute-sql`
    - adapters for `SQLite`, `PostgreSQL`, and `SQL Server / Azure SQL`
    - SQL tree-sitter in query buffers
    - `Ctrl+c Ctrl+c` execution chord
    - oil-style tables buffer
    - secure remembered connections using OS secret storage
  - **v1.1**
    - table and column autocomplete
    - schema cache refresh commands
    - better result formatting and large-result paging
    - improved statement detection and current-statement execution
    - session-only fallback behavior for platforms without usable keyring
  - **v2.0**
    - richer schema browser for columns, views, and indexes
    - query history
    - saved query buffers/snippets
    - row explorer
    - optional encrypted file fallback for unsupported secret-store environments

- **Technical Risks**:
  - Cross-engine differences in metadata discovery, SQL dialects, and connection-string parsing.
  - SQL LSP support quality may vary by dialect/backend.
  - Linux secret-store availability is inconsistent and may complicate “remember connection” UX.
  - Long-running or destructive queries require careful async execution and error handling.
  - Azure SQL and generic SQL Server support may require engine-specific driver behavior beyond shared SQL abstractions.

## Open Design Decisions

- Whether `db.show-connections` should also use oil-style UI or a picker-style buffer.
- Exact execution scope semantics for `Ctrl+c Ctrl+c` when no selection exists:
  - current statement
  - current paragraph
  - whole buffer
- Whether `db.show-tables` should later support opening generated `SELECT TOP/LIMIT` preview buffers directly from the oil-style listing.
