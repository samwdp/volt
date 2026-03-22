use editor_core::{Section, SectionAction, SectionItem, SectionTree};
use editor_git::{GitStatusSnapshot, StatusEntry};
use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

pub const GIT_STATUS_KIND: &str = "git-status";
pub const GIT_COMMIT_KIND: &str = "git-commit";
pub const HOOK_GIT_STATUS_OPEN_POPUP: &str = "ui.git.status-open-popup";
pub const ACTION_STAGE_FILE: &str = "git.stage-file";
pub const ACTION_STAGE_ALL: &str = "git.stage-all";
pub const ACTION_COMMIT_OPEN: &str = "git.commit-open";
pub const ACTION_PUSH: &str = "git.push";
pub const SECTION_HEADERS: &str = "git.status.headers";
pub const SECTION_IN_PROGRESS: &str = "git.status.in-progress";
pub const SECTION_STAGED: &str = "git.status.staged";
pub const SECTION_UNSTAGED: &str = "git.status.unstaged";
pub const SECTION_UNTRACKED: &str = "git.status.untracked";
pub const SECTION_STASHES: &str = "git.status.stashes";
pub const SECTION_UNPULLED: &str = "git.status.unpulled";
pub const SECTION_UNPUSHED: &str = "git.status.unpushed";
pub const SECTION_COMMIT: &str = "git.status.commit";

/// Returns the metadata for the git workflow package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "git",
        true,
        "Magit-style git workflows surfaced as buffers.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "git.status",
            "Opens the git status buffer.",
            vec![PluginAction::open_buffer(
                "*git-status*",
                GIT_STATUS_KIND,
                Some("Git Status"),
            )],
        ),
        PluginCommand::new(
            "git.status-open",
            "Opens the git status buffer.",
            vec![PluginAction::open_buffer(
                "*git-status*",
                GIT_STATUS_KIND,
                Some("Git Status"),
            )],
        ),
        PluginCommand::new(
            "git.status-open-popup",
            "Opens the git status buffer in the popup window.",
            vec![PluginAction::emit_hook(
                HOOK_GIT_STATUS_OPEN_POPUP,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "git.commit-open",
            "Opens the git commit buffer.",
            vec![PluginAction::open_buffer(
                "*git-commit*",
                GIT_COMMIT_KIND,
                Some("Git Commit"),
            )],
        ),
        PluginCommand::new(
            "git.branches",
            "Opens the git branches popup buffer.",
            vec![PluginAction::open_buffer(
                "*git-branches*",
                GIT_STATUS_KIND,
                Some("Git Branches"),
            )],
        ),
    ])
}

/// Builds the git status section tree for rendering.
pub fn status_sections(snapshot: &GitStatusSnapshot) -> SectionTree {
    let mut sections = Vec::new();
    sections.push(status_header_section(snapshot));
    if !snapshot.in_progress().is_empty() {
        sections.push(in_progress_section(snapshot));
    }
    sections.push(staged_section(snapshot));
    sections.push(unstaged_section(snapshot));
    sections.push(untracked_section(snapshot));
    if !snapshot.stashes().is_empty() {
        sections.push(stashes_section(snapshot));
    }
    sections.push(unpulled_section(snapshot));
    sections.push(unpushed_section(snapshot));
    sections.push(commit_section(snapshot));
    SectionTree::new(sections)
}

/// Returns the default commit buffer template.
pub fn commit_buffer_template() -> Vec<String> {
    vec![
        "# Write the commit message below. Lines starting with # are ignored.".to_owned(),
        "".to_owned(),
    ]
}

fn status_header_section(snapshot: &GitStatusSnapshot) -> Section {
    let mut items = Vec::new();
    let head_line = match (snapshot.branch(), snapshot.head()) {
        (Some(branch), Some(head)) => format!("Head: {branch} {} {}", head.hash(), head.summary()),
        (Some(branch), None) => format!("Head: {branch}"),
        (None, Some(head)) => format!("Head: {} {}", head.hash(), head.summary()),
        (None, None) => "Head: <unknown>".to_owned(),
    };
    items.push(SectionItem::new(head_line));
    if let Some(upstream) = snapshot.upstream() {
        let line = format!(
            "Upstream: {upstream} (ahead {}, behind {})",
            snapshot.ahead(),
            snapshot.behind()
        );
        items.push(SectionItem::new(line));
    }
    if let Some(push_remote) = snapshot.push_remote() {
        items.push(SectionItem::new(format!("Push: {push_remote}")));
    }
    Section::new(SECTION_HEADERS, "Status").with_items(items)
}

fn in_progress_section(snapshot: &GitStatusSnapshot) -> Section {
    let items = snapshot
        .in_progress()
        .iter()
        .map(|line| SectionItem::new(line.to_owned()))
        .collect::<Vec<_>>();
    Section::new(SECTION_IN_PROGRESS, "In progress").with_items(items)
}

fn staged_section(snapshot: &GitStatusSnapshot) -> Section {
    let items = snapshot
        .staged()
        .iter()
        .map(|entry| SectionItem::new(status_entry_label(entry, true)))
        .collect::<Vec<_>>();
    section_with_placeholder(
        SECTION_STAGED,
        format!("Staged changes ({})", snapshot.staged().len()),
        items,
    )
}

fn unstaged_section(snapshot: &GitStatusSnapshot) -> Section {
    let items = snapshot
        .unstaged()
        .iter()
        .map(|entry| {
            let action = SectionAction::new(ACTION_STAGE_FILE).with_detail(entry.path());
            SectionItem::new(status_entry_label(entry, false)).with_action(action)
        })
        .collect::<Vec<_>>();
    section_with_placeholder(
        SECTION_UNSTAGED,
        format!("Unstaged changes ({})", snapshot.unstaged().len()),
        items,
    )
}

fn untracked_section(snapshot: &GitStatusSnapshot) -> Section {
    let items = snapshot
        .untracked()
        .iter()
        .map(|path| {
            let action = SectionAction::new(ACTION_STAGE_FILE).with_detail(path);
            SectionItem::new(format!("untracked {path}")).with_action(action)
        })
        .collect::<Vec<_>>();
    section_with_placeholder(
        SECTION_UNTRACKED,
        format!("Untracked files ({})", snapshot.untracked().len()),
        items,
    )
}

fn stashes_section(snapshot: &GitStatusSnapshot) -> Section {
    let items = snapshot
        .stashes()
        .iter()
        .map(|entry| SectionItem::new(format!("{} {}", entry.name(), entry.summary())))
        .collect::<Vec<_>>();
    Section::new(
        SECTION_STASHES,
        format!("Stashes ({})", snapshot.stashes().len()),
    )
    .with_items(items)
}

fn unpulled_section(snapshot: &GitStatusSnapshot) -> Section {
    let (title, entries) = if snapshot.upstream().is_some() {
        (
            format!(
                "Unpulled from {} ({})",
                snapshot.upstream().unwrap_or("<upstream>"),
                snapshot.unpulled().len()
            ),
            snapshot.unpulled(),
        )
    } else {
        (
            format!("Recent commits ({})", snapshot.recent().len()),
            snapshot.recent(),
        )
    };
    let items = entries
        .iter()
        .map(|entry| SectionItem::new(format!("{} {}", entry.hash(), entry.summary())))
        .collect::<Vec<_>>();
    section_with_placeholder(SECTION_UNPULLED, title, items)
}

fn unpushed_section(snapshot: &GitStatusSnapshot) -> Section {
    let (title, entries) = if snapshot.upstream().is_some() {
        (
            format!(
                "Unpushed to {} ({})",
                snapshot.upstream().unwrap_or("<upstream>"),
                snapshot.unpushed().len()
            ),
            snapshot.unpushed(),
        )
    } else {
        (
            format!("Recent commits ({})", snapshot.recent().len()),
            snapshot.recent(),
        )
    };
    let items = entries
        .iter()
        .map(|entry| SectionItem::new(format!("{} {}", entry.hash(), entry.summary())))
        .collect::<Vec<_>>();
    section_with_placeholder(SECTION_UNPUSHED, title, items)
}

fn commit_section(snapshot: &GitStatusSnapshot) -> Section {
    let item = if snapshot.staged().is_empty() {
        SectionItem::new("No staged changes to commit.")
    } else {
        SectionItem::new("Press c to commit staged changes.")
            .with_action(SectionAction::new(ACTION_COMMIT_OPEN))
    };
    Section::new(SECTION_COMMIT, "Commit").with_items(vec![item])
}

fn status_entry_label(entry: &StatusEntry, staged: bool) -> String {
    let code = if staged {
        entry.index_status()
    } else {
        entry.worktree_status()
    };
    let label = match code {
        'A' => "added",
        'D' => "deleted",
        'M' => "modified",
        'R' => "renamed",
        'C' => "copied",
        'U' => "updated",
        _ => "changed",
    };
    format!("{label} {}", entry.path())
}

fn section_with_placeholder(id: &str, title: String, items: Vec<SectionItem>) -> Section {
    let items = if items.is_empty() {
        vec![SectionItem::new("(none)")]
    } else {
        items
    };
    Section::new(id, title).with_items(items)
}
