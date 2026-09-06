    use super::command_builds_user_library;

    #[test]
    fn detects_explicit_volt_user_build_commands() {
        assert!(command_builds_user_library("cargo build -p volt-user"));
        assert!(command_builds_user_library(
            "cargo build -p volt -p volt-user"
        ));
        assert!(command_builds_user_library("cargo test -p volt-user"));
    }

    #[test]
    fn ignores_non_user_or_non_cargo_commands() {
        assert!(!command_builds_user_library("cargo build -p volt"));
        assert!(!command_builds_user_library("cargo xtask ci"));
        assert!(!command_builds_user_library("dotnet build volt-user"));
    }
