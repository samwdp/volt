# User Source Tree and Plugin SDK boundary

Status: Accepted (target). Staging still copies engine crates into `user/vendor/` while the Plugin SDK path-depends on `crates/editor-*`. That vendoring is a stopgap, not the authoring contract.

Release builds ship a User Source Tree beside the binary so end users can rebuild the User Library (edit Builtin Plugins, add Plugin Packages, pull crates.io deps) without the full Volt repository. Rebuild happens in that tree (`cargo build --release -p volt-user`), not in the Volt git checkout.

Staging copies user sources and rewrites manifests into a standalone Cargo workspace by inlining inherited package fields (including table-form `workspace = true`). The staged tree must not contain a `vendor/` copy of Volt engine crates once the Plugin SDK is a leaf crate: crates.io plus its own sources only. High-level editor concepts used by Plugin Packages (packages, commands, hooks, keybindings, themes, language specs, picker layout, sections, and similar authoring types) live in the SDK. Host and core crates depend on the SDK and adapt those types to engine internals.

Builtin Plugin modules author only against the Plugin SDK (plus crates.io). The User Library manifest path-depends on that SDK plus crates.io. Offline/airgapped registry vendoring of crates.io is not required.
