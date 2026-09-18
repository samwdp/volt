# User Source Tree and Plugin SDK boundary

Status: Accepted.

Release builds ship a User Source Tree beside the binary so end users can rebuild the User Library (edit Builtin Plugins, add Plugin Packages, pull crates.io deps) without the full Volt repository. Rebuild happens in that tree (`cargo build --release -p volt-user`), not in the Volt git checkout.

Staging copies user sources and rewrites manifests into a standalone Cargo workspace by inlining inherited package fields (including table-form `workspace = true`). The staged tree does not contain a `vendor/` copy of Volt engine crates: the Plugin SDK is a leaf crate (crates.io plus its own sources only). High-level editor concepts used by Plugin Packages (packages, commands, hooks, keybindings, themes, language specs, picker layout, sections, and similar authoring types) live in the SDK. Host and core crates depend on the SDK and adapt those types to engine internals.

Builtin Plugin modules author only against the Plugin SDK (plus crates.io). The User Library manifest path-depends on that SDK plus crates.io. Offline/airgapped registry vendoring of crates.io is not required.

New host capabilities for Plugin Packages go through the in-process Volt API function table (`volt::buf`, `volt::lsp`, `volt::ui`, `volt::hook`) installed by the host into the compiled User Library (rlib). `UserLibraryModule` is at the abi_stable prefix-field limit (`keymap_config_v1` is `last_prefix_field`); new capabilities must not add prefix fields. Hot reload must load each staged cdylib via `load_user_library_module_from_path` rather than `UserLibraryModuleRef::load_from_file`, because the latter sticks to the first mapped module for the process lifetime. Hot-reloaded cdylib copies still fall back to host-side workers until a non-prefix seam exists.
