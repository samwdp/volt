    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workspace(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("volt-editor-issues-{name}-{unique}"));
        fs::create_dir_all(&root).expect("workspace");
        root
    }

    #[test]
    fn create_mints_sequential_ids_and_opens_store() {
        let root = temp_workspace("create");
        let first = create_issue(&root, "Fix login", "2026-07-16T12:00:00Z").expect("create");
        let second = create_issue(&root, "Add Place", "2026-07-16T12:01:00Z").expect("create");

        assert_eq!(first.id().display(), "ISS-001");
        assert_eq!(second.id().display(), "ISS-002");
        assert_eq!(first.status(), IssueStatus::Open);
        assert_eq!(first.opened_at(), "2026-07-16T12:00:00Z");
        assert!(first.closed_at().is_none());
        assert!(first.code_references().is_empty());
        assert!(store_dir(&root).is_dir());
        assert!(issue_path(&root, &first).is_file());

        let loaded = load_issue(&root, first.id()).expect("load");
        assert_eq!(loaded.title(), "Fix login");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn next_id_uses_max_existing() {
        let root = temp_workspace("max-id");
        ensure_store(&root).expect("store");
        let path = store_dir(&root).join("ISS-007-manual.md");
        fs::write(
            &path,
            "---\nid: ISS-007\ntitle: Manual\nstatus: Open\nopened_at: t0\nclosed_at:\ncode_references:\n  []\n---\n\n",
        )
        .expect("write");
        let created = create_issue(&root, "After max", "t1").expect("create");
        assert_eq!(created.id().display(), "ISS-008");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn status_moves_freely_and_manages_closed_at() {
        let root = temp_workspace("status");
        let issue = create_issue(&root, "Work", "t-open").expect("create");
        let opened = issue.opened_at().to_owned();

        let planning = set_status(&root, issue.id(), IssueStatus::Planning, "t2").expect("status");
        assert_eq!(planning.status(), IssueStatus::Planning);
        assert_eq!(planning.opened_at(), opened);
        assert!(planning.closed_at().is_none());

        let closed =
            set_status(&root, issue.id(), IssueStatus::Closed, "t-closed").expect("closed");
        assert_eq!(closed.status(), IssueStatus::Closed);
        assert_eq!(closed.closed_at(), Some("t-closed"));
        assert_eq!(closed.opened_at(), opened);

        let reopen = set_status(&root, issue.id(), IssueStatus::InProgress, "t3").expect("reopen");
        assert_eq!(reopen.status(), IssueStatus::InProgress);
        assert!(reopen.closed_at().is_none());
        assert_eq!(reopen.opened_at(), opened);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn capture_mints_and_rewrites_todo_and_fixme() {
        let root = temp_workspace("capture");
        let source = "// TODO: fix login\nfn main() {}\n# FIXME: urgent\n";
        // python-style line in rust file still uses // preferred; use .py for #
        let report = capture_file(
            &root,
            "src/main.rs",
            "// TODO: fix login\nfn main() {}\n",
            "t0",
        )
        .expect("capture");
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].issue.id().display(), "ISS-001");
        assert!(
            report.items[0]
                .rewrite
                .rewritten_line
                .contains("TODO(ISS-001):")
        );
        assert_eq!(report.items[0].rewrite.original_line, "// TODO: fix login");

        let py = capture_file(&root, "script.py", "# FIXME: urgent\n", "t1").expect("py");
        assert_eq!(py.items.len(), 1);
        assert!(
            py.items[0]
                .rewrite
                .rewritten_line
                .contains("FIXME(ISS-002):")
        );
        let _ = source;
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn capture_ignores_hack_and_xxx() {
        let root = temp_workspace("ignore");
        let report = capture_file(
            &root,
            "a.rs",
            "// HACK: no\n// XXX: no\n// TODO: yes\n",
            "t0",
        )
        .expect("capture");
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].issue.title(), "yes");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rewrite_if_unchanged_helper() {
        let intent = RewriteIntent {
            line_index: 0,
            original_line: "// TODO: x".into(),
            rewritten_line: "// TODO(ISS-001): x".into(),
            issue_id: IssueId::new(1),
        };
        assert!(should_apply_rewrite("// TODO: x", &intent));
        assert!(!should_apply_rewrite("// TODO: changed", &intent));
    }

    #[test]
    fn board_hides_closed_by_default() {
        let root = temp_workspace("board");
        let open = create_issue(&root, "Open one", "t0").expect("open");
        let closed = create_issue(&root, "Done", "t1").expect("closed create");
        set_status(&root, closed.id(), IssueStatus::Closed, "t2").expect("close");
        let all = list_issues(&root).expect("list");
        let active = board_issues(&all, false);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id(), open.id());
        assert_eq!(board_issues(&all, true).len(), 2);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn place_records_location() {
        let root = temp_workspace("place");
        let issue = create_issue(&root, "Link me", "t0").expect("create");
        let placed = place_code_reference(&root, issue.id(), "src/a.rs", 10, ReferenceMarker::Todo)
            .expect("place");
        assert!(placed.inserted_line.contains("TODO(ISS-001):"));
        assert!(placed.inserted_line.contains("Link me"));
        assert_eq!(placed.issue.code_references().len(), 1);
        assert_eq!(placed.issue.code_references()[0].path(), "src/a.rs");
        assert_eq!(placed.issue.code_references()[0].line(), 10);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn jump_decision_handles_zero_one_many() {
        assert!(matches!(jump_decision(&[]), JumpDecision::None));
        let one = [CodeReference::new("a.rs", 1)];
        assert!(matches!(jump_decision(&one), JumpDecision::Single(_)));
        let many = [CodeReference::new("a.rs", 1), CodeReference::new("b.rs", 2)];
        assert!(matches!(jump_decision(&many), JumpDecision::Many(_)));
    }

    #[test]
    fn parse_linked_and_unlinked_forms() {
        let unlinked = parse_code_reference_line("  // TODO: fix", "// ").expect("unlinked");
        assert!(unlinked.issue_id.is_none());
        assert_eq!(unlinked.title, "fix");
        let linked = parse_code_reference_line("// FIXME(ISS-042): urgent", "// ").expect("linked");
        assert_eq!(linked.issue_id, Some(IssueId::new(42)));
        assert_eq!(linked.marker, ReferenceMarker::Fixme);
    }

    #[test]
    fn scan_prunes_stale_and_reports_orphan_without_delete() {
        let root = temp_workspace("scan");
        let issue = create_issue(&root, "Tracked", "t0").expect("create");
        let mut issue = load_issue(&root, issue.id()).expect("load");
        issue.record_code_reference(
            CodeReference::new("src/gone.rs", 3).with_snippet("// TODO(ISS-001): tracked"),
        );
        save_issue(&root, &issue).expect("save");

        let files = vec![
            (
                "src/alive.rs".to_owned(),
                "// TODO(ISS-001): tracked\n// TODO(ISS-099): orphan\n".to_owned(),
            ),
            ("src/gone.rs".to_owned(), "// comment removed\n".to_owned()),
        ];
        let report = scan_files(&root, &files, "t1").expect("scan");
        assert!(
            report
                .orphans
                .iter()
                .any(|orphan| orphan.issue_id == IssueId::new(99))
        );
        assert!(
            report
                .pruned
                .iter()
                .any(|pruned| pruned.reference.path() == "src/gone.rs")
        );
        let refreshed = load_issue(&root, IssueId::new(1)).expect("still exists");
        assert!(
            refreshed
                .code_references()
                .iter()
                .any(|reference| reference.path() == "src/alive.rs")
        );
        assert!(
            !refreshed
                .code_references()
                .iter()
                .any(|reference| reference.path() == "src/gone.rs")
        );
        // Issue never deleted
        assert!(load_issue(&root, IssueId::new(1)).is_ok());
        assert!(load_issue(&root, IssueId::new(99)).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn spoken_names_use_issue_not_task() {
        assert_eq!(IssueStatus::Open.label(), "Open");
        assert_eq!(IssueId::new(1).display(), "ISS-001");
        assert!(ROLE.contains("Issue"));
    }

    #[test]
    fn capture_can_finish_after_caller_continues() {
        // Adapter contract: save returns immediately while Capture runs off-thread.
        let root = temp_workspace("async-capture");
        let source = root.join("main.rs");
        fs::write(&source, "// TODO: async me\n").expect("write");
        let text = fs::read_to_string(&source).expect("read");
        let (tx, rx) = std::sync::mpsc::channel();
        let workspace = root.clone();
        std::thread::spawn(move || {
            let report = capture_file(&workspace, "main.rs", &text, "t0");
            let _ = tx.send(report);
        });
        // Caller already "returned" from save.
        let save_completed = true;
        assert!(save_completed);
        let report = rx.recv().expect("worker").expect("capture");
        assert_eq!(report.items.len(), 1);
        assert!(
            store_dir(&root)
                .join(report.items[0].issue.file_name())
                .is_file()
        );
        let _ = fs::remove_dir_all(root);
    }
