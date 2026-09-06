    use super::*;

    #[test]
    fn lsp_diagnostic_scope_matches_active_workspace_root() {
        let active_root = Some(Path::new("P:\\volt"));
        let active_buffer_paths = HashSet::new();
        assert!(lsp_diagnostic_belongs_to_workspace(
            active_root,
            Some(Path::new("P:\\volt")),
            &active_buffer_paths,
            Path::new("P:\\volt\\src\\main.rs"),
        ));
        assert!(!lsp_diagnostic_belongs_to_workspace(
            active_root,
            Some(Path::new("P:\\volt\\nested")),
            &active_buffer_paths,
            Path::new("P:\\volt\\nested\\src\\main.rs"),
        ));
    }

    #[test]
    fn lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root() {
        let diagnostic_path = PathBuf::from("scratch.rs");
        let active_buffer_paths = HashSet::from([diagnostic_path.clone()]);
        assert!(lsp_diagnostic_belongs_to_workspace(
            None,
            None,
            &active_buffer_paths,
            &diagnostic_path,
        ));
        assert!(!lsp_diagnostic_belongs_to_workspace(
            None,
            None,
            &HashSet::new(),
            Path::new("other.rs"),
        ));
    }
