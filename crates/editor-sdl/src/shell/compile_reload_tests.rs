    use super::{
        DynamicUserLibrary, cargo_command_selects_other_package, command_builds_user_library,
        stage_user_library_for_reload, user_library_filename, validate_runtime_user_library,
    };
    use abi_stable::library::RootModule;
    use std::{env, fs, path::PathBuf};

    #[test]
    fn detects_explicit_volt_user_build_commands() {
        assert!(command_builds_user_library("cargo build -p volt-user"));
        assert!(command_builds_user_library(
            "cargo build -p volt -p volt-user"
        ));
        assert!(command_builds_user_library("cargo test -p volt-user"));
        assert!(command_builds_user_library(
            "cargo build --release -p volt-user"
        ));
    }

    #[test]
    fn ignores_non_user_or_non_cargo_commands() {
        assert!(!command_builds_user_library("cargo build -p volt"));
        assert!(!command_builds_user_library("cargo xtask ci"));
        assert!(!command_builds_user_library("dotnet build volt-user"));
        assert!(!command_builds_user_library("cargo build"));
    }

    #[test]
    fn package_flag_detection_ignores_volt_user_and_flags_others() {
        assert!(!cargo_command_selects_other_package(
            "cargo build --release -p volt-user"
        ));
        assert!(!cargo_command_selects_other_package("cargo build"));
        assert!(cargo_command_selects_other_package("cargo build -p volt"));
        assert!(cargo_command_selects_other_package(
            "cargo build --package=editor-sdl"
        ));
    }

    fn workspace_debug_user_library() -> Option<PathBuf> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.ancestors().nth(2)?;
        let path = workspace_root
            .join("target")
            .join("debug")
            .join(user_library_filename());
        path.is_file().then_some(path)
    }

    #[test]
    fn stages_unique_hot_copy_and_loads_user_library() {
        let Some(built) = workspace_debug_user_library() else {
            // Workspace may not have built volt-user yet in isolated crates.
            return;
        };
        let first = stage_user_library_for_reload(&built).expect("stage first copy");
        let second = stage_user_library_for_reload(&built).expect("stage second copy");
        assert_ne!(first, second);
        assert!(first
            .parent()
            .is_some_and(|parent| parent.ends_with("volt-user-hot")));
        let library = DynamicUserLibrary::load_from_file(&first).expect("load staged library");
        validate_runtime_user_library(library.as_ref()).expect("validate staged library");
        // Original artifact remains writable after staging (Windows DLL lock workaround).
        let marker = built.with_extension("hot-reload-probe");
        fs::write(&marker, b"ok").expect("write beside mapped original");
        let _ = fs::remove_file(marker);
    }

    #[test]
    fn release_user_dll_exports_edited_picker_height() {
        let path = PathBuf::from(r"c:\tools\volt\release\user\target\release\user.dll");
        if !path.is_file() {
            return;
        }
        let library = DynamicUserLibrary::load_from_file(&path).expect("load release user.dll");
        let layout = library.picker_layout();
        assert!(
            (layout.height_fraction - 0.5).abs() < f32::EPSILON,
            "release user.dll picker height should be 0.5 after edit, got {}",
            layout.height_fraction
        );
    }

    #[test]
    fn hot_reload_loader_maps_distinct_staged_copies() {
        let path = PathBuf::from(r"c:\tools\volt\release\user\target\release\user.dll");
        if !path.is_file() {
            return;
        }
        let first_path = stage_user_library_for_reload(&path).expect("stage first");
        let second_path = stage_user_library_for_reload(&path).expect("stage second");
        let first = editor_plugin_api::load_user_library_module_from_path(&first_path)
            .expect("load first staged copy");
        let second = editor_plugin_api::load_user_library_module_from_path(&second_path)
            .expect("load second staged copy");
        let first_fn = first.pane_config_v1() as usize;
        let second_fn = second.pane_config_v1() as usize;
        assert_ne!(
            first_fn, second_fn,
            "each staged copy must map its own pane_config export"
        );
        assert!(
            ((first.pane_config_v1())().picker_layout().height_fraction - 0.5).abs()
                < f32::EPSILON
        );
        assert!(
            ((second.pane_config_v1())().picker_layout().height_fraction - 0.5).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn root_module_load_from_file_is_sticky_to_first_path() {
        let path = PathBuf::from(r"c:\tools\volt\release\user\target\release\user.dll");
        if !path.is_file() {
            return;
        }
        let first_path = stage_user_library_for_reload(&path).expect("stage first");
        let second_path = stage_user_library_for_reload(&path).expect("stage second");
        let first = editor_plugin_api::abi::UserLibraryModuleRef::load_from_file(&first_path)
            .expect("first sticky load");
        let second = editor_plugin_api::abi::UserLibraryModuleRef::load_from_file(&second_path)
            .expect("second sticky load");
        assert_eq!(
            first.pane_config_v1() as usize,
            second.pane_config_v1() as usize,
            "abi_stable RootModule::load_from_file must stay sticky (documents why hot reload needs the non-sticky helper)"
        );
    }
