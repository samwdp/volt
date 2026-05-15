use crate::icon_font::symbols::md;
use editor_plugin_api::{
    LanguageServerRootStrategy, LanguageServerSpec, PluginAction, PluginCommand, PluginHookBinding,
    PluginHookDeclaration, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
};

pub const HOOK_LSP_START: &str = "lsp.server-start";
pub const HOOK_LSP_STOP: &str = "lsp.server-stop";
pub const HOOK_LSP_RESTART: &str = "lsp.server-restart";
pub const HOOK_LSP_LOG: &str = "lsp.open-log";
pub const HOOK_LSP_DEFINITION: &str = "lsp.goto-definition";
pub const HOOK_LSP_REFERENCES: &str = "lsp.goto-references";
pub const HOOK_LSP_IMPLEMENTATION: &str = "lsp.goto-implementation";
pub const HOOK_LSP_DIAGNOSTICS: &str = "lsp.diagnostics";
pub const HOOK_LSP_CODE_ACTIONS: &str = "lsp.code-actions";
pub const HOOK_LSP_COPILOT_SIGN_IN: &str = "lsp.copilot-sign-in";
pub const HOOK_LSP_COPILOT_SIGN_OUT: &str = "lsp.copilot-sign-out";
pub const CODE_ACTIONS_CHORD: &str = "Ctrl+Space";
pub const COPILOT_LANGUAGE_SERVER: &str = "copilot-language-server";
pub const COPILOT_ENABLED_DEFAULT: bool = false;
pub const SERVER_RUST_ANALYZER: &str = "rust-analyzer";
pub const SERVER_MARKSMAN: &str = "marksman";
pub const SERVER_CSHARP_LS: &str = "csharp-ls";
pub const SERVER_TYPESCRIPT_LANGUAGE_SERVER: &str = "typescript-language-server";
pub const SERVER_TAILWINDCSS_LANGUAGE_SERVER: &str = "tailwindcss-language-server";
pub const SERVER_VSCODE_JSON_LANGUAGE_SERVER: &str = "vscode-json-language-server";
pub const SERVER_VSCODE_HTML_LANGUAGE_SERVER: &str = "vscode-html-language-server";
pub const SERVER_VSCODE_CSS_LANGUAGE_SERVER: &str = "vscode-css-language-server";
pub const SERVER_CLANGD: &str = "clangd";
pub const SERVER_PYRIGHT_LANGSERVER: &str = "jedi-language-server";
pub const SERVER_MAKEFILE_LANGUAGE_SERVER: &str = "makefile-language-server";
pub const SERVER_ZLS: &str = "zls";
pub const SERVER_GOPLS: &str = "gopls";
pub const SERVER_SQLS: &str = "sqls";
pub const SERVER_OLS: &str = "ols";
pub const SERVER_TOMBI: &str = "tombi";
pub const SERVER_YAML_LANGUAGE_SERVER: &str = "yaml-language-server";
pub const SERVER_BASH_LANGUAGE_SERVER: &str = "bash-language-server";
pub const SERVER_CMAKE_LANGUAGE_SERVER: &str = "cmake-language-server";
pub const SERVER_GRAPHQL_LANGUAGE_SERVICE: &str = "graphql-language-service";
pub const SERVER_TERRAFORM_LS: &str = "terraform-ls";
pub const SERVER_JDTLS: &str = "jdtls";
pub const SERVER_KOTLIN_LANGUAGE_SERVER: &str = "kotlin-language-server";
pub const SERVER_LUA_LANGUAGE_SERVER: &str = "lua-language-server";
pub const SERVER_NIL: &str = "nil";
pub const SERVER_PERLNAVIGATOR: &str = "perlnavigator";
pub const SERVER_INTELEPHENSE: &str = "intelephense";
pub const SERVER_R_LANGUAGE_SERVER: &str = "r-language-server";
pub const SERVER_RUBY_LSP: &str = "ruby-lsp";
pub const SERVER_METALS: &str = "metals";
pub const SERVER_SOURCEKIT_LSP: &str = "sourcekit-lsp";
pub const SERVER_TEXLAB: &str = "texlab";
pub const SERVER_SOLC_LSP: &str = "solc-lsp";
pub const SERVER_ELIXIR_LS: &str = "elixir-ls";
pub const SERVER_CLOJURE_LSP: &str = "clojure-lsp";
pub const SERVER_BUFLS: &str = "bufls";
pub const SERVER_XML_LANGUAGE_SERVER: &str = "xml-language-server";
pub const SHOW_BUFFER_DIAGNOSTICS: bool = true;
pub const DIAGNOSTIC_LINE_LIMIT: usize = 8;
pub const DIAGNOSTIC_ICON: &str = md::MD_ALERT_CIRCLE_OUTLINE;

#[cfg(windows)]
const ELIXIR_LS_PROGRAM: &str = "language_server.bat";

#[cfg(not(windows))]
const ELIXIR_LS_PROGRAM: &str = "language_server.sh";

/// Returns the metadata for the LSP integration package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lsp",
        true,
        "Language server integration, lifecycle commands, and startup hooks.",
    )
    .with_commands(vec![
        hook_command(
            "lsp.start",
            "Starts the language servers registered for the active file.",
            HOOK_LSP_START,
            None,
        ),
        hook_command(
            "lsp.stop",
            "Stops the language servers attached to the active file.",
            HOOK_LSP_STOP,
            None,
        ),
        hook_command(
            "lsp.restart",
            "Restarts the language servers for the active file.",
            HOOK_LSP_RESTART,
            None,
        ),
        hook_command(
            "lsp.log",
            "Opens the live LSP transport log buffer.",
            HOOK_LSP_LOG,
            None,
        ),
        hook_command(
            "lsp.definition",
            "Jumps to the LSP definition under the cursor.",
            HOOK_LSP_DEFINITION,
            None,
        ),
        hook_command(
            "lsp.references",
            "Finds LSP references for the symbol under the cursor.",
            HOOK_LSP_REFERENCES,
            None,
        ),
        hook_command(
            "lsp.implementation",
            "Jumps to LSP implementations for the symbol under the cursor.",
            HOOK_LSP_IMPLEMENTATION,
            None,
        ),
        hook_command(
            "lsp.diagnostics",
            "Opens a picker of current LSP diagnostics from live servers.",
            HOOK_LSP_DIAGNOSTICS,
            None,
        ),
        hook_command(
            "lsp.code-actions",
            "Opens LSP code actions available at the cursor.",
            HOOK_LSP_CODE_ACTIONS,
            None,
        ),
        hook_command(
            "lsp.code-action",
            "Opens LSP code actions available at the cursor.",
            HOOK_LSP_CODE_ACTIONS,
            None,
        ),
        hook_command(
            "lsp.copilot-sign-in",
            "Starts GitHub Copilot device authentication for the active file.",
            HOOK_LSP_COPILOT_SIGN_IN,
            None,
        ),
        hook_command(
            "lsp.copilot-sign-out",
            "Signs the active GitHub Copilot language server session out.",
            HOOK_LSP_COPILOT_SIGN_OUT,
            None,
        ),
        hook_command(
            "lsp.start-rust-analyzer",
            "Starts rust-analyzer for the active Rust file.",
            HOOK_LSP_START,
            Some(SERVER_RUST_ANALYZER),
        ),
        hook_command(
            "lsp.start-marksman",
            "Starts marksman for the active Markdown file.",
            HOOK_LSP_START,
            Some(SERVER_MARKSMAN),
        ),
        hook_command(
            "lsp.start-csharp-ls",
            "Starts csharp-ls for the active C# or Razor file.",
            HOOK_LSP_START,
            Some(SERVER_CSHARP_LS),
        ),
        hook_command(
            "lsp.start-typescript-language-server",
            "Starts typescript-language-server for the active TS/TSX/JS/JSX file.",
            HOOK_LSP_START,
            Some(SERVER_TYPESCRIPT_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-tailwindcss-language-server",
            "Starts tailwindcss-language-server for the active HTML/JS/JSX/TS/TSX file when Tailwind project markers are present.",
            HOOK_LSP_START,
            Some(SERVER_TAILWINDCSS_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-vscode-json-language-server",
            "Starts vscode-json-language-server for the active JSON file.",
            HOOK_LSP_START,
            Some(SERVER_VSCODE_JSON_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-vscode-html-language-server",
            "Starts vscode-html-language-server for the active HTML file.",
            HOOK_LSP_START,
            Some(SERVER_VSCODE_HTML_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-vscode-css-language-server",
            "Starts vscode-css-language-server for the active CSS or SCSS file.",
            HOOK_LSP_START,
            Some(SERVER_VSCODE_CSS_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-clangd",
            "Starts clangd for the active C or C++ file.",
            HOOK_LSP_START,
            Some(SERVER_CLANGD),
        ),
        hook_command(
            "lsp.start-pyright-langserver",
            "Starts pyright-langserver for the active Python file.",
            HOOK_LSP_START,
            Some(SERVER_PYRIGHT_LANGSERVER),
        ),
        hook_command(
            "lsp.start-makefile-language-server",
            "Starts makefile-language-server for the active Make file.",
            HOOK_LSP_START,
            Some(SERVER_MAKEFILE_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-zls",
            "Starts zls for the active Zig file.",
            HOOK_LSP_START,
            Some(SERVER_ZLS),
        ),
        hook_command(
            "lsp.start-gopls",
            "Starts gopls for the active Go file.",
            HOOK_LSP_START,
            Some(SERVER_GOPLS),
        ),
        hook_command(
            "lsp.start-sqls",
            "Starts sqls for the active SQL file.",
            HOOK_LSP_START,
            Some(SERVER_SQLS),
        ),
        hook_command(
            "lsp.start-ols",
            "Starts ols for the active Odin file.",
            HOOK_LSP_START,
            Some(SERVER_OLS),
        ),
        hook_command(
            "lsp.start-tombi",
            "Starts tombi for the active TOML file.",
            HOOK_LSP_START,
            Some(SERVER_TOMBI),
        ),
        hook_command(
            "lsp.start-yaml-language-server",
            "Starts yaml-language-server for the active YAML file.",
            HOOK_LSP_START,
            Some(SERVER_YAML_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-bash-language-server",
            "Starts bash-language-server for the active shell script.",
            HOOK_LSP_START,
            Some(SERVER_BASH_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-cmake-language-server",
            "Starts cmake-language-server for the active CMake file.",
            HOOK_LSP_START,
            Some(SERVER_CMAKE_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-graphql-language-service",
            "Starts graphql-language-service for the active GraphQL file.",
            HOOK_LSP_START,
            Some(SERVER_GRAPHQL_LANGUAGE_SERVICE),
        ),
        hook_command(
            "lsp.start-terraform-ls",
            "Starts terraform-ls for the active HCL or Terraform file.",
            HOOK_LSP_START,
            Some(SERVER_TERRAFORM_LS),
        ),
        hook_command(
            "lsp.start-jdtls",
            "Starts jdtls for the active Java file.",
            HOOK_LSP_START,
            Some(SERVER_JDTLS),
        ),
        hook_command(
            "lsp.start-kotlin-language-server",
            "Starts kotlin-language-server for the active Kotlin file.",
            HOOK_LSP_START,
            Some(SERVER_KOTLIN_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-lua-language-server",
            "Starts lua-language-server for the active Lua file.",
            HOOK_LSP_START,
            Some(SERVER_LUA_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-nil",
            "Starts nil for the active Nix file.",
            HOOK_LSP_START,
            Some(SERVER_NIL),
        ),
        hook_command(
            "lsp.start-perlnavigator",
            "Starts perlnavigator for the active Perl file.",
            HOOK_LSP_START,
            Some(SERVER_PERLNAVIGATOR),
        ),
        hook_command(
            "lsp.start-intelephense",
            "Starts intelephense for the active PHP file.",
            HOOK_LSP_START,
            Some(SERVER_INTELEPHENSE),
        ),
        hook_command(
            "lsp.start-r-language-server",
            "Starts r-language-server for the active R file.",
            HOOK_LSP_START,
            Some(SERVER_R_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-ruby-lsp",
            "Starts ruby-lsp for the active Ruby file.",
            HOOK_LSP_START,
            Some(SERVER_RUBY_LSP),
        ),
        hook_command(
            "lsp.start-metals",
            "Starts metals for the active Scala file.",
            HOOK_LSP_START,
            Some(SERVER_METALS),
        ),
        hook_command(
            "lsp.start-sourcekit-lsp",
            "Starts sourcekit-lsp for the active Swift file.",
            HOOK_LSP_START,
            Some(SERVER_SOURCEKIT_LSP),
        ),
        hook_command(
            "lsp.start-texlab",
            "Starts texlab for the active LaTeX file.",
            HOOK_LSP_START,
            Some(SERVER_TEXLAB),
        ),
        hook_command(
            "lsp.start-solc-lsp",
            "Starts solc-lsp for the active Solidity file.",
            HOOK_LSP_START,
            Some(SERVER_SOLC_LSP),
        ),
        hook_command(
            "lsp.start-elixir-ls",
            "Starts elixir-ls for the active Elixir file.",
            HOOK_LSP_START,
            Some(SERVER_ELIXIR_LS),
        ),
        hook_command(
            "lsp.start-clojure-lsp",
            "Starts clojure-lsp for the active Clojure file.",
            HOOK_LSP_START,
            Some(SERVER_CLOJURE_LSP),
        ),
        hook_command(
            "lsp.start-bufls",
            "Starts bufls for the active Protocol Buffers file.",
            HOOK_LSP_START,
            Some(SERVER_BUFLS),
        ),
        hook_command(
            "lsp.start-xml-language-server",
            "Starts xml-language-server for the active XML file.",
            HOOK_LSP_START,
            Some(SERVER_XML_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-copilot-language-server",
            "Starts copilot-language-server for the active file.",
            HOOK_LSP_START,
            Some(COPILOT_LANGUAGE_SERVER),
        ),
    ])
    .with_hook_declarations(vec![
        PluginHookDeclaration::new(
            HOOK_LSP_START,
            "Runs after an LSP start command is triggered.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_STOP,
            "Runs after an LSP stop command is triggered.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_RESTART,
            "Runs after an LSP restart command is triggered.",
        ),
        PluginHookDeclaration::new(HOOK_LSP_LOG, "Opens the live LSP transport log buffer."),
        PluginHookDeclaration::new(
            HOOK_LSP_DEFINITION,
            "Navigates to the LSP definition under the cursor.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_REFERENCES,
            "Lists LSP references for the symbol under the cursor.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_IMPLEMENTATION,
            "Navigates to LSP implementations for the symbol under the cursor.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_DIAGNOSTICS,
            "Opens a picker of current LSP diagnostics from live servers.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_CODE_ACTIONS,
            "Opens LSP code actions available at the cursor.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_COPILOT_SIGN_IN,
            "Starts GitHub Copilot authentication for the active buffer.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_COPILOT_SIGN_OUT,
            "Signs the active GitHub Copilot language server session out.",
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new(
            CODE_ACTIONS_CHORD,
            "lsp.code-actions",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
    ])
    .with_hook_bindings(vec![
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rust",
            "lsp.start",
            Some(".rs"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-markdown",
            "lsp.start",
            Some(".md"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-markdown-long",
            "lsp.start",
            Some(".markdown"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-csharp",
            "lsp.start",
            Some(".cs"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-razor",
            "lsp.start",
            Some(".razor"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cshtml",
            "lsp.start",
            Some(".cshtml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-typescript",
            "lsp.start",
            Some(".ts"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-tsx",
            "lsp.start",
            Some(".tsx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-javascript",
            "lsp.start",
            Some(".js"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-jsx",
            "lsp.start",
            Some(".jsx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-json",
            "lsp.start",
            Some(".json"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-html",
            "lsp.start",
            Some(".html"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-htm",
            "lsp.start",
            Some(".htm"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-css",
            "lsp.start",
            Some(".css"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-scss",
            "lsp.start",
            Some(".scss"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-c",
            "lsp.start",
            Some(".c"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-header",
            "lsp.start",
            Some(".h"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cc",
            "lsp.start",
            Some(".cc"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cpp",
            "lsp.start",
            Some(".cpp"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cxx",
            "lsp.start",
            Some(".cxx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-hpp",
            "lsp.start",
            Some(".hpp"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-hh",
            "lsp.start",
            Some(".hh"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-hxx",
            "lsp.start",
            Some(".hxx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-python",
            "lsp.start",
            Some(".py"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-mk",
            "lsp.start",
            Some(".mk"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-mak",
            "lsp.start",
            Some(".mak"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-make",
            "lsp.start",
            Some(".make"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-makefile",
            "lsp.start",
            Some("Makefile"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-gnu-makefile",
            "lsp.start",
            Some("GNUmakefile"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-makefile-lower",
            "lsp.start",
            Some("makefile"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-zig",
            "lsp.start",
            Some(".zig"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-go",
            "lsp.start",
            Some(".go"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-sql",
            "lsp.start",
            Some(".sql"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-odin",
            "lsp.start",
            Some(".odin"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-toml",
            "lsp.start",
            Some(".toml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-yaml",
            "lsp.start",
            Some(".yaml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-yml",
            "lsp.start",
            Some(".yml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-sh",
            "lsp.start",
            Some(".sh"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-bash",
            "lsp.start",
            Some(".bash"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-zsh",
            "lsp.start",
            Some(".zsh"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-ksh",
            "lsp.start",
            Some(".ksh"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-ash",
            "lsp.start",
            Some(".ash"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-dash",
            "lsp.start",
            Some(".dash"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-mksh",
            "lsp.start",
            Some(".mksh"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cmake",
            "lsp.start",
            Some(".cmake"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cmakelists",
            "lsp.start",
            Some("CMakeLists.txt"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-gql",
            "lsp.start",
            Some(".gql"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-graphql",
            "lsp.start",
            Some(".graphql"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-graphqls",
            "lsp.start",
            Some(".graphqls"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-hcl",
            "lsp.start",
            Some(".hcl"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-tf",
            "lsp.start",
            Some(".tf"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-nomad",
            "lsp.start",
            Some(".nomad"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-java",
            "lsp.start",
            Some(".java"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-jav",
            "lsp.start",
            Some(".jav"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-pde",
            "lsp.start",
            Some(".pde"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-kt",
            "lsp.start",
            Some(".kt"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-kts",
            "lsp.start",
            Some(".kts"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-lua",
            "lsp.start",
            Some(".lua"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rockspec",
            "lsp.start",
            Some(".rockspec"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-nix",
            "lsp.start",
            Some(".nix"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-pl",
            "lsp.start",
            Some(".pl"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-pm",
            "lsp.start",
            Some(".pm"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-t",
            "lsp.start",
            Some(".t"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-psgi",
            "lsp.start",
            Some(".psgi"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-php",
            "lsp.start",
            Some(".php"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-inc",
            "lsp.start",
            Some(".inc"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-php4",
            "lsp.start",
            Some(".php4"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-php5",
            "lsp.start",
            Some(".php5"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-phtml",
            "lsp.start",
            Some(".phtml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-ctp",
            "lsp.start",
            Some(".ctp"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-r",
            "lsp.start",
            Some(".r"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rb",
            "lsp.start",
            Some(".rb"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rake",
            "lsp.start",
            Some(".rake"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-irb",
            "lsp.start",
            Some(".irb"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-gemspec",
            "lsp.start",
            Some(".gemspec"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rabl",
            "lsp.start",
            Some(".rabl"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-jbuilder",
            "lsp.start",
            Some(".jbuilder"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-jb",
            "lsp.start",
            Some(".jb"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-podspec",
            "lsp.start",
            Some(".podspec"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rjs",
            "lsp.start",
            Some(".rjs"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rbi",
            "lsp.start",
            Some(".rbi"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rbs",
            "lsp.start",
            Some(".rbs"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-scala",
            "lsp.start",
            Some(".scala"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-sbt",
            "lsp.start",
            Some(".sbt"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-sc",
            "lsp.start",
            Some(".sc"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-swift",
            "lsp.start",
            Some(".swift"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-swiftinterface",
            "lsp.start",
            Some(".swiftinterface"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-tex",
            "lsp.start",
            Some(".tex"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-dtx",
            "lsp.start",
            Some(".dtx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-ins",
            "lsp.start",
            Some(".ins"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-sty",
            "lsp.start",
            Some(".sty"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cls",
            "lsp.start",
            Some(".cls"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rd",
            "lsp.start",
            Some(".rd"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-bbx",
            "lsp.start",
            Some(".bbx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cbx",
            "lsp.start",
            Some(".cbx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-sol",
            "lsp.start",
            Some(".sol"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-ex",
            "lsp.start",
            Some(".ex"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-exs",
            "lsp.start",
            Some(".exs"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-clj",
            "lsp.start",
            Some(".clj"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cljs",
            "lsp.start",
            Some(".cljs"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-cljc",
            "lsp.start",
            Some(".cljc"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-edn",
            "lsp.start",
            Some(".edn"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-proto",
            "lsp.start",
            Some(".proto"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-xml",
            "lsp.start",
            Some(".xml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-svg",
            "lsp.start",
            Some(".svg"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-xsd",
            "lsp.start",
            Some(".xsd"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-xslt",
            "lsp.start",
            Some(".xslt"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-xsl",
            "lsp.start",
            Some(".xsl"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rng",
            "lsp.start",
            Some(".rng"),
        ),
    ])
}

/// Returns LSP server specifications compiled into the user library.
pub fn language_servers() -> Vec<LanguageServerSpec> {
    let mut servers = vec![
        LanguageServerSpec::new(
            SERVER_RUST_ANALYZER,
            "rust",
            ["rs"],
            "rust-analyzer",
            std::iter::empty::<&str>(),
        )
        .with_root_markers(["Cargo.toml", "rust-project.json"]),
        LanguageServerSpec::new(
            SERVER_MARKSMAN,
            "markdown",
            ["md", "markdown"],
            "marksman",
            ["server"],
        ),
        LanguageServerSpec::new(
            SERVER_CSHARP_LS,
            "csharp",
            ["cs", "razor", "cshtml"],
            "csharp-ls",
            std::iter::empty::<&str>(),
        )
        .with_document_language_ids([("razor", "razor"), ("cshtml", "razor")])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["*.sln", "*.csproj"])
        .with_workspace_configuration(
            "csharp",
            LanguageServerSpec::workspace_settings_object([
                ("logLevel", "info".into()),
                ("applyFormattingOptions", false.into()),
                ("analyzersEnabled", true.into()),
                ("useMetadataUris", true.into()),
                ("razorSupport", true.into()),
                (
                    "solutionPathOverride",
                    LanguageServerSpec::workspace_settings_null(),
                ),
                ("locale", LanguageServerSpec::workspace_settings_null()),
                (
                    "debug",
                    LanguageServerSpec::workspace_settings_object([
                        (
                            "solutionLoadDelay",
                            LanguageServerSpec::workspace_settings_null(),
                        ),
                        ("debugMode", false.into()),
                    ]),
                ),
            ]),
        ),
        LanguageServerSpec::new(
            SERVER_TYPESCRIPT_LANGUAGE_SERVER,
            "typescript",
            ["ts", "tsx", "js", "jsx"],
            "typescript-language-server",
            ["--stdio"],
        )
        .with_document_language_ids([
            ("tsx", "typescriptreact"),
            ("js", "javascript"),
            ("jsx", "javascriptreact"),
        ])
        .with_root_markers([
            "package.json",
            "tsconfig.json",
            "jsconfig.json",
            "deno.json",
            "deno.jsonc",
        ]),
        LanguageServerSpec::new(
            SERVER_TAILWINDCSS_LANGUAGE_SERVER,
            "html",
            ["html", "js", "jsx", "ts", "tsx"],
            SERVER_TAILWINDCSS_LANGUAGE_SERVER,
            ["--stdio"],
        )
        .with_document_language_ids([
            ("js", "javascript"),
            ("jsx", "javascriptreact"),
            ("ts", "typescript"),
            ("tsx", "typescriptreact"),
        ])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "tailwind.config.js",
            "tailwind.config.cjs",
            "tailwind.config.mjs",
            "tailwind.config.ts",
            "tailwind.config.cts",
            "tailwind.config.mts",
            "node_modules/tailwindcss/package.json",
        ])
        .with_activation_markers([
            "tailwind.config.js",
            "tailwind.config.cjs",
            "tailwind.config.mjs",
            "tailwind.config.ts",
            "tailwind.config.cts",
            "tailwind.config.mts",
            "node_modules/tailwindcss/package.json",
        ]),
        LanguageServerSpec::new(
            SERVER_VSCODE_JSON_LANGUAGE_SERVER,
            "json",
            ["json"],
            "vscode-json-language-server",
            ["--stdio"],
        ),
        LanguageServerSpec::new(
            SERVER_VSCODE_HTML_LANGUAGE_SERVER,
            "html",
            ["html", "htm"],
            "vscode-html-language-server",
            ["--stdio"],
        )
        .with_root_markers([
            "package.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "bun.lock",
            ".git",
        ]),
        LanguageServerSpec::new(
            SERVER_VSCODE_CSS_LANGUAGE_SERVER,
            "css",
            ["css", "scss"],
            "vscode-css-language-server",
            ["--stdio"],
        )
        .with_document_language_ids([("scss", "scss")])
        .with_root_markers([
            "package.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "bun.lock",
            ".git",
        ]),
        LanguageServerSpec::new(
            SERVER_CLANGD,
            "cpp",
            ["c", "h", "cc", "cpp", "cxx", "hpp", "hh", "hxx"],
            "clangd",
            std::iter::empty::<&str>(),
        )
        .with_document_language_ids([("c", "c"), ("h", "c")])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "compile_commands.json",
            "compile_flags.txt",
            ".clangd",
            "CMakeLists.txt",
            "meson.build",
            "configure.ac",
        ]),
        LanguageServerSpec::new(
            SERVER_PYRIGHT_LANGSERVER,
            "python",
            ["py"],
            "jedi-language-server",
            [""],
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "pyproject.toml",
            "pyrightconfig.json",
            "setup.py",
            "setup.cfg",
            "requirements.txt",
            "Pipfile",
        ]),
        LanguageServerSpec::new(
            SERVER_MAKEFILE_LANGUAGE_SERVER,
            "make",
            ["mk", "mak", "make"],
            "makefile-language-server",
            ["--stdio"],
        )
        .with_file_names(["Makefile", "GNUmakefile", "makefile"])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["Makefile", "GNUmakefile", "makefile"]),
        LanguageServerSpec::new(
            SERVER_ZLS,
            "zig",
            ["zig"],
            "zls",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["zls.json", "build.zig", "build.zig.zon"]),
        LanguageServerSpec::new(
            SERVER_GOPLS,
            "go",
            ["go"],
            "gopls",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["go.work", "go.mod", "go.sum"]),
        LanguageServerSpec::new(
            SERVER_SQLS,
            "sql",
            ["sql"],
            "sqls",
            std::iter::empty::<&str>(),
        )
        .with_workspace_configuration_section("sqls")
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([".sqls.json", ".sqls.yaml", ".sqls.yml"]),
        LanguageServerSpec::new(
            SERVER_OLS,
            "odin",
            ["odin"],
            "ols",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["ols.json", "ols.toml"]),
        LanguageServerSpec::new(SERVER_TOMBI, "toml", ["toml"], "tombi", ["lsp"]),
        LanguageServerSpec::new(
            SERVER_YAML_LANGUAGE_SERVER,
            "yaml",
            ["yaml", "yml"],
            "yaml-language-server",
            ["--stdio"],
        ),
        LanguageServerSpec::new(
            SERVER_BASH_LANGUAGE_SERVER,
            "bash",
            ["sh", "bash", "zsh", "ksh", "ash", "dash", "mksh"],
            "bash-language-server",
            ["start"],
        ),
        LanguageServerSpec::new(
            SERVER_CMAKE_LANGUAGE_SERVER,
            "cmake",
            ["cmake"],
            "cmake-language-server",
            std::iter::empty::<&str>(),
        )
        .with_file_names(["CMakeLists.txt"])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["CMakeLists.txt", "CMakePresets.json", "*.cmake"]),
        LanguageServerSpec::new(
            SERVER_GRAPHQL_LANGUAGE_SERVICE,
            "graphql",
            ["gql", "graphql", "graphqls"],
            "graphql-lsp",
            ["server", "-m", "stream"],
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "package.json",
            ".graphqlrc",
            "graphql.config.js",
            "graphql.config.ts",
        ]),
        LanguageServerSpec::new(
            SERVER_TERRAFORM_LS,
            "hcl",
            ["hcl", "tf", "nomad"],
            "terraform-ls",
            ["serve"],
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["*.tf", "terragrunt.hcl", ".terraform.lock.hcl"]),
        LanguageServerSpec::new(
            SERVER_JDTLS,
            "java",
            ["java", "jav", "pde"],
            "jdtls",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["pom.xml", "build.gradle", "build.gradle.kts"]),
        LanguageServerSpec::new(
            SERVER_KOTLIN_LANGUAGE_SERVER,
            "kotlin",
            ["kt", "kts"],
            "kotlin-language-server",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "settings.gradle",
            "settings.gradle.kts",
            "build.gradle",
            "build.gradle.kts",
        ]),
        LanguageServerSpec::new(
            SERVER_LUA_LANGUAGE_SERVER,
            "lua",
            ["lua", "rockspec"],
            "lua-language-server",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            ".luarc.json",
            ".luacheckrc",
            ".stylua.toml",
            "selene.toml",
            ".git",
        ]),
        LanguageServerSpec::new(
            SERVER_NIL,
            "nix",
            ["nix"],
            "nil",
            std::iter::empty::<&str>(),
        ),
        LanguageServerSpec::new(
            SERVER_PERLNAVIGATOR,
            "perl",
            ["pl", "pm", "t", "psgi"],
            "perlnavigator",
            ["--stdio"],
        ),
        LanguageServerSpec::new(
            SERVER_INTELEPHENSE,
            "php",
            ["php", "inc", "php4", "php5", "phtml", "ctp"],
            "intelephense",
            ["--stdio"],
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["composer.json", "index.php"]),
        LanguageServerSpec::new(
            SERVER_R_LANGUAGE_SERVER,
            "r",
            ["r"],
            "R",
            ["--no-echo", "-e", "languageserver::run()"],
        ),
        LanguageServerSpec::new(
            SERVER_RUBY_LSP,
            "ruby",
            [
                "rb", "rake", "irb", "gemspec", "rabl", "jbuilder", "jb", "podspec", "rjs", "rbi",
                "rbs",
            ],
            "ruby-lsp",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["Gemfile", ".ruby-version"]),
        LanguageServerSpec::new(
            SERVER_METALS,
            "scala",
            ["scala", "sbt", "sc"],
            "metals",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "build.sbt",
            "build.sc",
            "build.gradle",
            "build.gradle.kts",
            "pom.xml",
            ".scala-build",
        ]),
        LanguageServerSpec::new(
            SERVER_SOURCEKIT_LSP,
            "swift",
            ["swift", "swiftinterface"],
            "sourcekit-lsp",
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["Package.swift"]),
        LanguageServerSpec::new(
            SERVER_TEXLAB,
            "latex",
            ["tex", "dtx", "ins", "sty", "cls", "rd", "bbx", "cbx"],
            "texlab",
            std::iter::empty::<&str>(),
        ),
        LanguageServerSpec::new(SERVER_SOLC_LSP, "solidity", ["sol"], "solc", ["--lsp"])
            .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
            .with_root_markers([
                "foundry.toml",
                "hardhat.config.js",
                "hardhat.config.ts",
                "truffle-config.js",
                "truffle-config.ts",
                "brownie-config.yaml",
                "brownie-config.yml",
            ]),
        LanguageServerSpec::new(
            SERVER_ELIXIR_LS,
            "elixir",
            ["ex", "exs"],
            ELIXIR_LS_PROGRAM,
            std::iter::empty::<&str>(),
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers(["mix.exs", ".formatter.exs"]),
        LanguageServerSpec::new(
            SERVER_CLOJURE_LSP,
            "clojure",
            ["clj", "cljs", "cljc", "edn"],
            "clojure-lsp",
            std::iter::empty::<&str>(),
        )
        .with_document_language_ids([("edn", "edn")])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "deps.edn",
            "project.clj",
            "bb.edn",
            "build.boot",
            "shadow-cljs.edn",
        ]),
        LanguageServerSpec::new(SERVER_BUFLS, "proto", ["proto"], "bufls", ["serve"])
            .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
            .with_root_markers(["buf.yaml", "buf.work.yaml", "buf.gen.yaml", "buf.lock"]),
        LanguageServerSpec::new(
            SERVER_XML_LANGUAGE_SERVER,
            "xml",
            ["xml", "svg", "xsd", "xslt", "xsl", "rng"],
            "xml-language-server",
            ["--stdio"],
        ),
    ];
    servers.push(copilot_language_server(&servers));
    servers
}

fn copilot_language_server(servers: &[LanguageServerSpec]) -> LanguageServerSpec {
    let mut file_extensions = Vec::new();
    let mut file_names = Vec::new();
    let mut document_language_ids = Vec::new();
    for server in servers {
        for extension in server.file_extensions() {
            if !file_extensions.iter().any(|existing| existing == extension) {
                file_extensions.push(extension.clone());
                document_language_ids.push((
                    extension.clone(),
                    server
                        .document_language_id_for_extension(extension)
                        .to_owned(),
                ));
            }
        }
        for file_name in server.file_names() {
            if !file_names.iter().any(|existing| existing == file_name) {
                file_names.push(file_name.clone());
                document_language_ids.push((file_name.clone(), server.language_id().to_owned()));
            }
        }
    }

    LanguageServerSpec::new(
        COPILOT_LANGUAGE_SERVER,
        "plaintext",
        file_extensions,
        COPILOT_LANGUAGE_SERVER,
        ["--stdio"],
    )
    .with_file_names(file_names)
    .with_document_language_ids(document_language_ids)
    .with_enabled_by_default(COPILOT_ENABLED_DEFAULT)
}

fn hook_command(
    name: &str,
    description: &str,
    hook_name: &str,
    detail: Option<&str>,
) -> PluginCommand {
    let mut actions = Vec::new();
    if let Some(detail) = detail {
        actions.push(PluginAction::log_message(format!(
            "Starting language server `{detail}` from the user LSP package."
        )));
    }
    actions.push(PluginAction::emit_hook(hook_name, detail));
    PluginCommand::new(name, description, actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_plugin_api::PluginPackage;
    use std::collections::BTreeSet;

    fn has_command(package: &PluginPackage, name: &str) -> bool {
        package
            .commands()
            .iter()
            .any(|command| command.name() == name)
    }

    fn auto_start_binding_details(package: &PluginPackage) -> BTreeSet<String> {
        package
            .hook_bindings()
            .iter()
            .filter(|binding| {
                binding.hook_name() == "buffer.file-open" && binding.command_name() == "lsp.start"
            })
            .filter_map(|binding| binding.detail_filter().map(str::to_owned))
            .collect()
    }

    #[test]
    fn package_exports_generic_lsp_commands_and_code_action_binding() {
        let package = package();

        assert_eq!(package.name(), "lsp");
        for command_name in [
            "lsp.start",
            "lsp.stop",
            "lsp.restart",
            "lsp.log",
            "lsp.definition",
            "lsp.references",
            "lsp.implementation",
            "lsp.diagnostics",
            "lsp.code-actions",
            "lsp.code-action",
        ] {
            assert!(
                has_command(&package, command_name),
                "{command_name} missing"
            );
        }

        assert!(package.key_bindings().iter().any(|binding| {
            binding.command_name() == "lsp.code-actions"
                && binding.scope() == PluginKeymapScope::Workspace
                && binding.vim_mode() == PluginVimMode::Normal
        }));
    }

    #[test]
    fn language_servers_have_unique_ids_and_nonempty_programs() {
        let servers = language_servers();
        let mut ids = BTreeSet::new();

        for server in &servers {
            assert!(!server.id().is_empty());
            assert!(
                ids.insert(server.id().to_owned()),
                "duplicate language server `{}`",
                server.id()
            );
            assert!(!server.language_id().is_empty());
            assert!(!server.program().is_empty());
            assert!(
                server
                    .file_extensions()
                    .iter()
                    .all(|ext| !ext.trim().is_empty()),
                "server `{}` has an empty file extension",
                server.id()
            );
            assert!(
                server
                    .file_names()
                    .iter()
                    .all(|name| !name.trim().is_empty()),
                "server `{}` has an empty file name matcher",
                server.id()
            );
            assert!(
                server
                    .document_language_ids()
                    .values()
                    .all(|value| !value.trim().is_empty()),
                "server `{}` has an empty document language id",
                server.id()
            );
            assert!(
                server
                    .activation_markers()
                    .iter()
                    .all(|marker| !marker.trim().is_empty()),
                "server `{}` has an empty activation marker",
                server.id()
            );
        }

        if let Some(copilot) = servers
            .iter()
            .find(|server| server.id() == COPILOT_LANGUAGE_SERVER)
        {
            assert_eq!(copilot.enabled_by_default(), COPILOT_ENABLED_DEFAULT);
        }
    }

    #[test]
    fn sqls_server_uses_sqls_workspace_configuration_section() {
        let sqls = language_servers()
            .into_iter()
            .find(|server| server.id() == SERVER_SQLS)
            .expect("sqls server should be registered");
        assert_eq!(sqls.workspace_configuration_section(), Some("sqls"));
    }

    #[test]
    fn auto_start_bindings_match_registered_server_path_matchers() {
        let package = package();
        let servers = language_servers();
        let actual = auto_start_binding_details(&package);
        let expected = servers
            .iter()
            .filter(|server| server.id() != COPILOT_LANGUAGE_SERVER)
            .flat_map(|server| {
                server
                    .file_extensions()
                    .iter()
                    .map(|extension| format!(".{extension}"))
                    .chain(server.file_names().iter().cloned())
            })
            .collect::<BTreeSet<_>>();

        assert_eq!(actual, expected);
    }

    #[test]
    fn csharp_workspace_configuration_remains_well_formed_when_present() {
        if let Some(csharp) = language_servers()
            .into_iter()
            .find(|server| server.id() == SERVER_CSHARP_LS)
            && let Some(settings) = csharp.workspace_configuration_settings()
        {
            assert!(settings.as_object().is_some());
        }
    }

    #[test]
    fn tailwind_server_requires_project_markers_and_maps_web_language_ids() {
        let tailwind = language_servers()
            .into_iter()
            .find(|server| server.id() == SERVER_TAILWINDCSS_LANGUAGE_SERVER)
            .expect("tailwind server");

        assert_eq!(tailwind.program(), SERVER_TAILWINDCSS_LANGUAGE_SERVER);
        assert_eq!(tailwind.args(), ["--stdio"]);
        assert_eq!(tailwind.language_id(), "html");
        assert_eq!(
            tailwind.document_language_id_for_extension("js"),
            "javascript"
        );
        assert_eq!(
            tailwind.document_language_id_for_extension("jsx"),
            "javascriptreact"
        );
        assert_eq!(
            tailwind.document_language_id_for_extension("ts"),
            "typescript"
        );
        assert_eq!(
            tailwind.document_language_id_for_extension("tsx"),
            "typescriptreact"
        );
        assert!(
            tailwind
                .activation_markers()
                .iter()
                .any(|marker| marker == "tailwind.config.ts")
        );
        assert!(
            tailwind
                .activation_markers()
                .iter()
                .any(|marker| marker == "node_modules/tailwindcss/package.json")
        );
    }
}
