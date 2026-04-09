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
pub const HOOK_LSP_CODE_ACTIONS: &str = "lsp.code-actions";
pub const CODE_ACTIONS_CHORD: &str = "Ctrl+Enter";
pub const SERVER_RUST_ANALYZER: &str = "rust-analyzer";
pub const SERVER_MARKSMAN: &str = "marksman";
pub const SERVER_CSHARP_LS: &str = "csharp-ls";
pub const SERVER_TYPESCRIPT_LANGUAGE_SERVER: &str = "typescript-language-server";
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
            "Starts csharp-ls for the active C# file.",
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
            HOOK_LSP_CODE_ACTIONS,
            "Opens LSP code actions available at the cursor.",
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
    vec![
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
            ["cs"],
            "csharp-ls",
            ["--features", "razor-support,metadata-uris"],
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "*.sln",
            "*.csproj",
            "global.json",
            "Directory.Build.props",
            "Directory.Build.targets",
        ]),
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
    ]
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
    use editor_plugin_api::{LanguageServerSpec, PluginPackage};

    fn server_by_id<'a>(servers: &'a [LanguageServerSpec], id: &str) -> &'a LanguageServerSpec {
        servers
            .iter()
            .find(|server| server.id() == id)
            .unwrap_or_else(|| panic!("language server `{id}` missing"))
    }

    fn string_values(values: &[String]) -> Vec<&str> {
        values.iter().map(String::as_str).collect()
    }

    fn has_command(package: &PluginPackage, name: &str) -> bool {
        package
            .commands()
            .iter()
            .any(|command| command.name() == name)
    }

    fn has_auto_start_binding(package: &PluginPackage, detail: &str) -> bool {
        package.hook_bindings().iter().any(|binding| {
            binding.hook_name() == "buffer.file-open"
                && binding.command_name() == "lsp.start"
                && binding.detail_filter() == Some(detail)
        })
    }

    #[test]
    fn package_registers_rich_language_server_defaults() {
        let package = package();
        let servers = language_servers();
        let ids = servers.iter().map(|server| server.id()).collect::<Vec<_>>();

        assert_eq!(package.name(), "lsp");
        assert!(package.auto_load());
        assert_eq!(package.commands().len(), 45);
        assert_eq!(package.hook_bindings().len(), 107);
        assert_eq!(servers.len(), 36);
        for expected in [
            SERVER_RUST_ANALYZER,
            SERVER_MARKSMAN,
            SERVER_CSHARP_LS,
            SERVER_TYPESCRIPT_LANGUAGE_SERVER,
            SERVER_VSCODE_JSON_LANGUAGE_SERVER,
            SERVER_VSCODE_HTML_LANGUAGE_SERVER,
            SERVER_VSCODE_CSS_LANGUAGE_SERVER,
            SERVER_CLANGD,
            SERVER_PYRIGHT_LANGSERVER,
            SERVER_MAKEFILE_LANGUAGE_SERVER,
            SERVER_ZLS,
            SERVER_GOPLS,
            SERVER_SQLS,
            SERVER_OLS,
            SERVER_TOMBI,
            SERVER_YAML_LANGUAGE_SERVER,
            SERVER_BASH_LANGUAGE_SERVER,
            SERVER_CMAKE_LANGUAGE_SERVER,
            SERVER_GRAPHQL_LANGUAGE_SERVICE,
            SERVER_TERRAFORM_LS,
            SERVER_JDTLS,
            SERVER_KOTLIN_LANGUAGE_SERVER,
            SERVER_LUA_LANGUAGE_SERVER,
            SERVER_NIL,
            SERVER_PERLNAVIGATOR,
            SERVER_INTELEPHENSE,
            SERVER_R_LANGUAGE_SERVER,
            SERVER_RUBY_LSP,
            SERVER_METALS,
            SERVER_SOURCEKIT_LSP,
            SERVER_TEXLAB,
            SERVER_SOLC_LSP,
            SERVER_ELIXIR_LS,
            SERVER_CLOJURE_LSP,
            SERVER_BUFLS,
            SERVER_XML_LANGUAGE_SERVER,
        ] {
            assert!(ids.contains(&expected), "{expected} missing");
        }

        let rust = server_by_id(&servers, SERVER_RUST_ANALYZER);
        assert_eq!(rust.language_id(), "rust");
        assert!(
            rust.args().is_empty(),
            "rust-analyzer now speaks stdio without a `--stdio` flag"
        );

        let csharp = server_by_id(&servers, SERVER_CSHARP_LS);
        assert_eq!(
            csharp.args().iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["--features", "razor-support,metadata-uris"]
        );

        let typescript = server_by_id(&servers, SERVER_TYPESCRIPT_LANGUAGE_SERVER);
        assert_eq!(typescript.language_id(), "typescript");
        assert_eq!(
            typescript
                .file_extensions()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["ts", "tsx", "js", "jsx"]
        );
        assert_eq!(
            typescript
                .args()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["--stdio"]
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".ts"),
            "typescript"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".tsx"),
            "typescriptreact"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".js"),
            "javascript"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".jsx"),
            "javascriptreact"
        );

        let css = server_by_id(&servers, SERVER_VSCODE_CSS_LANGUAGE_SERVER);
        assert_eq!(css.language_id(), "css");
        assert_eq!(
            css.file_extensions()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["css", "scss"]
        );
        assert_eq!(
            css.args().iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["--stdio"]
        );
        assert_eq!(css.document_language_id_for_extension(".css"), "css");
        assert_eq!(css.document_language_id_for_extension(".scss"), "scss");

        let clangd = server_by_id(&servers, SERVER_CLANGD);
        assert_eq!(clangd.language_id(), "cpp");
        assert!(
            clangd.args().is_empty(),
            "clangd now speaks stdio without a `--stdio` flag"
        );
        assert_eq!(clangd.document_language_id_for_extension(".c"), "c");
        assert_eq!(clangd.document_language_id_for_extension(".h"), "c");
        assert_eq!(clangd.document_language_id_for_extension(".cpp"), "cpp");

        let python = server_by_id(&servers, SERVER_PYRIGHT_LANGSERVER);
        assert_eq!(python.language_id(), "python");
        assert_eq!(
            python.args().iter().map(String::as_str).collect::<Vec<_>>(),
            vec![""]
        );

        let make = server_by_id(&servers, SERVER_MAKEFILE_LANGUAGE_SERVER);
        assert_eq!(
            string_values(make.file_names()),
            vec!["Makefile", "GNUmakefile", "makefile"]
        );

        let go = server_by_id(&servers, SERVER_GOPLS);
        assert_eq!(go.language_id(), "go");
        assert!(go.args().is_empty());

        let yaml = server_by_id(&servers, SERVER_YAML_LANGUAGE_SERVER);
        assert_eq!(yaml.language_id(), "yaml");
        assert_eq!(
            yaml.file_extensions()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["yaml", "yml"]
        );
        assert_eq!(
            yaml.args().iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["--stdio"]
        );

        for command_name in [
            "lsp.stop",
            "lsp.restart",
            "lsp.log",
            "lsp.definition",
            "lsp.references",
            "lsp.implementation",
            "lsp.code-actions",
            "lsp.code-action",
        ] {
            assert!(
                has_command(&package, command_name),
                "{command_name} missing"
            );
        }
        assert_eq!(package.key_bindings().len(), 1);
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == CODE_ACTIONS_CHORD)
        );
    }

    #[test]
    fn package_registers_query_language_server_start_commands_and_auto_start_bindings() {
        let package = package();

        for command_name in [
            "lsp.start-bash-language-server",
            "lsp.start-cmake-language-server",
            "lsp.start-graphql-language-service",
            "lsp.start-terraform-ls",
            "lsp.start-jdtls",
            "lsp.start-kotlin-language-server",
            "lsp.start-lua-language-server",
            "lsp.start-nil",
            "lsp.start-perlnavigator",
            "lsp.start-intelephense",
            "lsp.start-r-language-server",
            "lsp.start-ruby-lsp",
            "lsp.start-metals",
            "lsp.start-sourcekit-lsp",
            "lsp.start-texlab",
            "lsp.start-solc-lsp",
            "lsp.start-elixir-ls",
            "lsp.start-clojure-lsp",
            "lsp.start-bufls",
            "lsp.start-xml-language-server",
        ] {
            assert!(
                has_command(&package, command_name),
                "{command_name} missing"
            );
        }

        for detail in [
            ".sh",
            ".bash",
            ".zsh",
            ".ksh",
            ".ash",
            ".dash",
            ".mksh",
            ".cmake",
            ".gql",
            ".graphql",
            ".graphqls",
            ".hcl",
            ".tf",
            ".nomad",
            ".java",
            ".jav",
            ".pde",
            ".kt",
            ".kts",
            ".lua",
            ".rockspec",
            ".nix",
            ".pl",
            ".pm",
            ".t",
            ".psgi",
            ".php",
            ".inc",
            ".php4",
            ".php5",
            ".phtml",
            ".ctp",
            ".r",
            ".rb",
            ".rake",
            ".irb",
            ".gemspec",
            ".rabl",
            ".jbuilder",
            ".jb",
            ".podspec",
            ".rjs",
            ".rbi",
            ".rbs",
            ".scala",
            ".sbt",
            ".sc",
            ".swift",
            ".swiftinterface",
            ".tex",
            ".dtx",
            ".ins",
            ".sty",
            ".cls",
            ".rd",
            ".bbx",
            ".cbx",
            ".sol",
            ".ex",
            ".exs",
            ".clj",
            ".cljs",
            ".cljc",
            ".edn",
            ".proto",
            ".xml",
            ".svg",
            ".xsd",
            ".xslt",
            ".xsl",
            ".rng",
            "Makefile",
            "GNUmakefile",
            "makefile",
            "CMakeLists.txt",
        ] {
            assert!(
                has_auto_start_binding(&package, detail),
                "missing auto-start binding for `{detail}`"
            );
        }
    }

    #[test]
    fn new_language_servers_expose_expected_program_args_and_workspace_roots() {
        let servers = language_servers();

        let bash = server_by_id(&servers, SERVER_BASH_LANGUAGE_SERVER);
        assert_eq!(bash.language_id(), "bash");
        assert_eq!(
            string_values(bash.file_extensions()),
            vec!["sh", "bash", "zsh", "ksh", "ash", "dash", "mksh"]
        );
        assert_eq!(bash.program(), "bash-language-server");
        assert_eq!(string_values(bash.args()), vec!["start"]);

        let cmake = server_by_id(&servers, SERVER_CMAKE_LANGUAGE_SERVER);
        assert_eq!(cmake.language_id(), "cmake");
        assert_eq!(cmake.program(), "cmake-language-server");
        assert!(cmake.args().is_empty());
        assert_eq!(string_values(cmake.file_names()), vec!["CMakeLists.txt"]);
        assert_eq!(
            cmake.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(cmake.root_markers()),
            vec!["CMakeLists.txt", "CMakePresets.json", "*.cmake"]
        );

        let graphql = server_by_id(&servers, SERVER_GRAPHQL_LANGUAGE_SERVICE);
        assert_eq!(graphql.language_id(), "graphql");
        assert_eq!(graphql.program(), "graphql-lsp");
        assert_eq!(
            string_values(graphql.args()),
            vec!["server", "-m", "stream"]
        );
        assert_eq!(
            graphql.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(graphql.root_markers()),
            vec![
                "package.json",
                ".graphqlrc",
                "graphql.config.js",
                "graphql.config.ts",
            ]
        );

        let terraform = server_by_id(&servers, SERVER_TERRAFORM_LS);
        assert_eq!(terraform.language_id(), "hcl");
        assert_eq!(terraform.program(), "terraform-ls");
        assert_eq!(string_values(terraform.args()), vec!["serve"]);
        assert_eq!(
            terraform.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(terraform.root_markers()),
            vec!["*.tf", "terragrunt.hcl", ".terraform.lock.hcl"]
        );

        let java = server_by_id(&servers, SERVER_JDTLS);
        assert_eq!(java.language_id(), "java");
        assert_eq!(java.program(), "jdtls");
        assert!(java.args().is_empty());
        assert_eq!(
            java.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(java.root_markers()),
            vec!["pom.xml", "build.gradle", "build.gradle.kts"]
        );

        let kotlin = server_by_id(&servers, SERVER_KOTLIN_LANGUAGE_SERVER);
        assert_eq!(kotlin.language_id(), "kotlin");
        assert_eq!(kotlin.program(), "kotlin-language-server");
        assert!(kotlin.args().is_empty());
        assert_eq!(
            kotlin.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(kotlin.root_markers()),
            vec![
                "settings.gradle",
                "settings.gradle.kts",
                "build.gradle",
                "build.gradle.kts",
            ]
        );

        let lua = server_by_id(&servers, SERVER_LUA_LANGUAGE_SERVER);
        assert_eq!(lua.language_id(), "lua");
        assert_eq!(lua.program(), "lua-language-server");
        assert!(lua.args().is_empty());
        assert_eq!(
            lua.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(lua.root_markers()),
            vec![
                ".luarc.json",
                ".luacheckrc",
                ".stylua.toml",
                "selene.toml",
                ".git",
            ]
        );

        let nix = server_by_id(&servers, SERVER_NIL);
        assert_eq!(nix.language_id(), "nix");
        assert_eq!(nix.program(), "nil");
        assert!(nix.args().is_empty());

        let perl = server_by_id(&servers, SERVER_PERLNAVIGATOR);
        assert_eq!(perl.language_id(), "perl");
        assert_eq!(perl.program(), "perlnavigator");
        assert_eq!(string_values(perl.args()), vec!["--stdio"]);

        let php = server_by_id(&servers, SERVER_INTELEPHENSE);
        assert_eq!(php.language_id(), "php");
        assert_eq!(php.program(), "intelephense");
        assert_eq!(string_values(php.args()), vec!["--stdio"]);
        assert_eq!(
            php.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(php.root_markers()),
            vec!["composer.json", "index.php"]
        );

        let r = server_by_id(&servers, SERVER_R_LANGUAGE_SERVER);
        assert_eq!(r.language_id(), "r");
        assert_eq!(r.program(), "R");
        assert_eq!(
            string_values(r.args()),
            vec!["--no-echo", "-e", "languageserver::run()"]
        );

        let ruby = server_by_id(&servers, SERVER_RUBY_LSP);
        assert_eq!(ruby.language_id(), "ruby");
        assert_eq!(ruby.program(), "ruby-lsp");
        assert!(ruby.args().is_empty());
        assert_eq!(
            ruby.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(ruby.root_markers()),
            vec!["Gemfile", ".ruby-version"]
        );

        let metals = server_by_id(&servers, SERVER_METALS);
        assert_eq!(metals.language_id(), "scala");
        assert_eq!(metals.program(), "metals");
        assert!(metals.args().is_empty());
        assert_eq!(
            metals.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(metals.root_markers()),
            vec![
                "build.sbt",
                "build.sc",
                "build.gradle",
                "build.gradle.kts",
                "pom.xml",
                ".scala-build",
            ]
        );

        let swift = server_by_id(&servers, SERVER_SOURCEKIT_LSP);
        assert_eq!(swift.language_id(), "swift");
        assert_eq!(swift.program(), "sourcekit-lsp");
        assert!(swift.args().is_empty());
        assert_eq!(
            swift.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(string_values(swift.root_markers()), vec!["Package.swift"]);

        let texlab = server_by_id(&servers, SERVER_TEXLAB);
        assert_eq!(texlab.language_id(), "latex");
        assert_eq!(
            string_values(texlab.file_extensions()),
            vec!["tex", "dtx", "ins", "sty", "cls", "rd", "bbx", "cbx"]
        );
        assert_eq!(texlab.program(), "texlab");
        assert!(texlab.args().is_empty());

        let solidity = server_by_id(&servers, SERVER_SOLC_LSP);
        assert_eq!(solidity.language_id(), "solidity");
        assert_eq!(string_values(solidity.file_extensions()), vec!["sol"]);
        assert_eq!(solidity.program(), "solc");
        assert_eq!(string_values(solidity.args()), vec!["--lsp"]);
        assert_eq!(
            solidity.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(solidity.root_markers()),
            vec![
                "foundry.toml",
                "hardhat.config.js",
                "hardhat.config.ts",
                "truffle-config.js",
                "truffle-config.ts",
                "brownie-config.yaml",
                "brownie-config.yml",
            ]
        );

        let elixir = server_by_id(&servers, SERVER_ELIXIR_LS);
        assert_eq!(elixir.language_id(), "elixir");
        assert_eq!(string_values(elixir.file_extensions()), vec!["ex", "exs"]);
        assert_eq!(elixir.program(), ELIXIR_LS_PROGRAM);
        assert!(elixir.args().is_empty());
        assert_eq!(
            elixir.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(elixir.root_markers()),
            vec!["mix.exs", ".formatter.exs"]
        );

        let clojure = server_by_id(&servers, SERVER_CLOJURE_LSP);
        assert_eq!(clojure.language_id(), "clojure");
        assert_eq!(
            string_values(clojure.file_extensions()),
            vec!["clj", "cljs", "cljc", "edn"]
        );
        assert_eq!(clojure.program(), "clojure-lsp");
        assert!(clojure.args().is_empty());
        assert_eq!(
            clojure.document_language_id_for_extension(".clj"),
            "clojure"
        );
        assert_eq!(
            clojure.document_language_id_for_extension(".cljs"),
            "clojure"
        );
        assert_eq!(
            clojure.document_language_id_for_extension(".cljc"),
            "clojure"
        );
        assert_eq!(clojure.document_language_id_for_extension(".edn"), "edn");
        assert_eq!(
            clojure.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(clojure.root_markers()),
            vec![
                "deps.edn",
                "project.clj",
                "bb.edn",
                "build.boot",
                "shadow-cljs.edn"
            ]
        );

        let proto = server_by_id(&servers, SERVER_BUFLS);
        assert_eq!(proto.language_id(), "proto");
        assert_eq!(string_values(proto.file_extensions()), vec!["proto"]);
        assert_eq!(proto.program(), "bufls");
        assert_eq!(string_values(proto.args()), vec!["serve"]);
        assert_eq!(
            proto.root_strategy(),
            LanguageServerRootStrategy::MarkersOrWorkspace
        );
        assert_eq!(
            string_values(proto.root_markers()),
            vec!["buf.yaml", "buf.work.yaml", "buf.gen.yaml", "buf.lock"]
        );

        let xml = server_by_id(&servers, SERVER_XML_LANGUAGE_SERVER);
        assert_eq!(xml.language_id(), "xml");
        assert_eq!(
            string_values(xml.file_extensions()),
            vec!["xml", "svg", "xsd", "xslt", "xsl", "rng"]
        );
        assert_eq!(xml.program(), "xml-language-server");
        assert_eq!(string_values(xml.args()), vec!["--stdio"]);
    }
}
