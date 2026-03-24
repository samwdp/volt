use editor_core::{Section, SectionAction, SectionItem, SectionTree};
use editor_git::{GitStatusSnapshot, StatusEntry};
use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

pub const GIT_STATUS_KIND: &str = "git-status";
pub const GIT_COMMIT_KIND: &str = "git-commit";
pub const GIT_DIFF_KIND: &str = "git-diff";
pub const GIT_LOG_KIND: &str = "git-log";
pub const GIT_STASH_KIND: &str = "git-stash";
pub const HOOK_GIT_STATUS_OPEN_POPUP: &str = "ui.git.status-open-popup";
pub const HOOK_GIT_DIFF_OPEN: &str = "ui.git.diff-open";
pub const HOOK_GIT_LOG_OPEN: &str = "ui.git.log-open";
pub const HOOK_GIT_STASH_LIST_OPEN: &str = "ui.git.stash-list-open";
pub const ACTION_STAGE_FILE: &str = "git.stage-file";
pub const ACTION_STAGE_ALL: &str = "git.stage-all";
pub const ACTION_UNSTAGE_FILE: &str = "git.unstage-file";
pub const ACTION_COMMIT_OPEN: &str = "git.commit-open";
pub const ACTION_PUSH: &str = "git.push";
pub const ACTION_SHOW_COMMIT: &str = "git.show-commit";
pub const ACTION_SHOW_STASH: &str = "git.show-stash";
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
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "git.status-open",
            "Opens the git status buffer.",
            vec![PluginAction::open_buffer(
                "*git-status*",
                GIT_STATUS_KIND,
                None::<&str>,
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
            "git.diff",
            "Opens the git diff buffer.",
            vec![PluginAction::emit_hook(HOOK_GIT_DIFF_OPEN, None::<&str>)],
        ),
        PluginCommand::new(
            "git.log",
            "Opens the git log buffer.",
            vec![PluginAction::emit_hook(HOOK_GIT_LOG_OPEN, None::<&str>)],
        ),
        PluginCommand::new(
            "git.stash-list",
            "Opens the git stash list buffer.",
            vec![PluginAction::emit_hook(
                HOOK_GIT_STASH_LIST_OPEN,
                None::<&str>,
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
    if let Some(section) = staged_section(snapshot) {
        sections.push(section);
    }
    if let Some(section) = unstaged_section(snapshot) {
        sections.push(section);
    }
    if let Some(section) = untracked_section(snapshot) {
        sections.push(section);
    }
    if !snapshot.stashes().is_empty() {
        sections.push(stashes_section(snapshot));
    }
    if let Some(section) = unpulled_section(snapshot) {
        sections.push(section);
    }
    if let Some(section) = unpushed_section(snapshot) {
        sections.push(section);
    }
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
    let branch_icon = crate::nerd_font::symbols::dev::DEV_GIT_BRANCH;
    let incoming_icon = crate::nerd_font::symbols::cod::COD_ARROW_DOWN;
    let outgoing_icon = crate::nerd_font::symbols::cod::COD_ARROW_UP;
    let head_line = match (snapshot.branch(), snapshot.head()) {
        (Some(branch), Some(head)) => format!(
            "{branch_icon} Head: {branch} {} {}",
            head.hash(),
            head.summary()
        ),
        (Some(branch), None) => format!("{branch_icon} Head: {branch}"),
        (None, Some(head)) => format!("{branch_icon} Head: {} {}", head.hash(), head.summary()),
        (None, None) => format!("{branch_icon} Head: <unknown>"),
    };
    items.push(SectionItem::new(head_line));
    if let Some(upstream) = snapshot.upstream() {
        let line = format!(
            "{incoming_icon} Upstream: {upstream} (ahead {}, behind {})",
            snapshot.ahead(),
            snapshot.behind()
        );
        items.push(SectionItem::new(line));
    }
    if let Some(push_remote) = snapshot.push_remote() {
        items.push(SectionItem::new(format!(
            "{outgoing_icon} Push: {push_remote}"
        )));
    }
    Section::new(
        SECTION_HEADERS,
        git_section_title(SECTION_HEADERS, "Status"),
    )
    .with_items(items)
}

fn in_progress_section(snapshot: &GitStatusSnapshot) -> Section {
    let items = snapshot
        .in_progress()
        .iter()
        .map(|line| {
            SectionItem::new(format!(
                "{} {line}",
                crate::nerd_font::symbols::cod::COD_LOADING
            ))
        })
        .collect::<Vec<_>>();
    Section::new(
        SECTION_IN_PROGRESS,
        git_section_title(SECTION_IN_PROGRESS, "In progress"),
    )
    .with_items(items)
}

fn staged_section(snapshot: &GitStatusSnapshot) -> Option<Section> {
    let items = snapshot
        .staged()
        .iter()
        .map(|entry| {
            let action = SectionAction::new(ACTION_UNSTAGE_FILE).with_detail(entry.path());
            SectionItem::new(status_entry_label(entry, true)).with_action(action)
        })
        .collect::<Vec<_>>();
    section_with_placeholder(
        SECTION_STAGED,
        git_section_title(
            SECTION_STAGED,
            format!("Staged changes ({})", snapshot.staged().len()),
        ),
        items,
    )
}

fn unstaged_section(snapshot: &GitStatusSnapshot) -> Option<Section> {
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
        git_section_title(
            SECTION_UNSTAGED,
            format!("Unstaged changes ({})", snapshot.unstaged().len()),
        ),
        items,
    )
}

fn untracked_section(snapshot: &GitStatusSnapshot) -> Option<Section> {
    let items = snapshot
        .untracked()
        .iter()
        .map(|path| {
            let action = SectionAction::new(ACTION_STAGE_FILE).with_detail(path);
            SectionItem::new(format!(
                "{} untracked {path}",
                crate::nerd_font::symbols::cod::COD_SYMBOL_FILE
            ))
            .with_action(action)
        })
        .collect::<Vec<_>>();
    section_with_placeholder(
        SECTION_UNTRACKED,
        git_section_title(
            SECTION_UNTRACKED,
            format!("Untracked files ({})", snapshot.untracked().len()),
        ),
        items,
    )
}

fn stashes_section(snapshot: &GitStatusSnapshot) -> Section {
    let items = snapshot
        .stashes()
        .iter()
        .map(|entry| {
            let action = SectionAction::new(ACTION_SHOW_STASH).with_detail(entry.name());
            SectionItem::new(format!(
                "{} {} {}",
                crate::nerd_font::symbols::cod::COD_HISTORY,
                entry.name(),
                entry.summary()
            ))
            .with_action(action)
        })
        .collect::<Vec<_>>();
    Section::new(
        SECTION_STASHES,
        git_section_title(
            SECTION_STASHES,
            format!("Stashes ({})", snapshot.stashes().len()),
        ),
    )
    .with_items(items)
}

fn unpulled_section(snapshot: &GitStatusSnapshot) -> Option<Section> {
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
        .map(|entry| {
            let action = SectionAction::new(ACTION_SHOW_COMMIT).with_detail(entry.hash());
            SectionItem::new(format!(
                "{} {} {}",
                crate::nerd_font::symbols::cod::COD_ARROW_DOWN,
                entry.hash(),
                entry.summary()
            ))
            .with_action(action)
        })
        .collect::<Vec<_>>();
    section_with_placeholder(
        SECTION_UNPULLED,
        git_section_title(SECTION_UNPULLED, title),
        items,
    )
}

fn unpushed_section(snapshot: &GitStatusSnapshot) -> Option<Section> {
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
        .map(|entry| {
            let action = SectionAction::new(ACTION_SHOW_COMMIT).with_detail(entry.hash());
            SectionItem::new(format!(
                "{} {} {}",
                crate::nerd_font::symbols::cod::COD_ARROW_UP,
                entry.hash(),
                entry.summary()
            ))
            .with_action(action)
        })
        .collect::<Vec<_>>();
    section_with_placeholder(
        SECTION_UNPUSHED,
        git_section_title(SECTION_UNPUSHED, title),
        items,
    )
}

fn commit_section(snapshot: &GitStatusSnapshot) -> Section {
    let item = if snapshot.staged().is_empty() {
        SectionItem::new(format!(
            "{} No staged changes to commit.",
            crate::nerd_font::symbols::cod::COD_GIT_COMMIT
        ))
    } else {
        SectionItem::new(format!(
            "{} Press c to commit staged changes.",
            crate::nerd_font::symbols::cod::COD_GIT_COMMIT
        ))
        .with_action(SectionAction::new(ACTION_COMMIT_OPEN))
    };
    Section::new(SECTION_COMMIT, git_section_title(SECTION_COMMIT, "Commit")).with_items(vec![item])
}

fn status_entry_label(entry: &StatusEntry, staged: bool) -> String {
    let code = if staged {
        entry.index_status()
    } else {
        entry.worktree_status()
    };
    let (icon, label) = match code {
        'A' => (crate::nerd_font::symbols::cod::COD_DIFF_ADDED, "added"),
        'D' => (crate::nerd_font::symbols::cod::COD_DIFF_REMOVED, "deleted"),
        'M' => (
            crate::nerd_font::symbols::cod::COD_DIFF_MODIFIED,
            "modified",
        ),
        'R' => (crate::nerd_font::symbols::cod::COD_DIFF_RENAMED, "renamed"),
        'C' => (crate::nerd_font::symbols::cod::COD_ARROW_SWAP, "copied"),
        'U' => (crate::nerd_font::symbols::cod::COD_SYNC, "updated"),
        _ => (crate::nerd_font::symbols::cod::COD_DIFF_MODIFIED, "changed"),
    };
    format!("{icon} {label} {}", entry.path())
}

fn git_section_title(id: &str, title: impl AsRef<str>) -> String {
    let icon = match id {
        SECTION_HEADERS => crate::nerd_font::symbols::dev::DEV_GIT_BRANCH,
        SECTION_IN_PROGRESS => crate::nerd_font::symbols::cod::COD_LOADING,
        SECTION_STAGED => crate::nerd_font::symbols::cod::COD_CHECK,
        SECTION_UNSTAGED => crate::nerd_font::symbols::cod::COD_DIFF_MODIFIED,
        SECTION_UNTRACKED => crate::nerd_font::symbols::cod::COD_SYMBOL_FILE,
        SECTION_STASHES => crate::nerd_font::symbols::cod::COD_HISTORY,
        SECTION_UNPULLED => crate::nerd_font::symbols::cod::COD_ARROW_DOWN,
        SECTION_UNPUSHED => crate::nerd_font::symbols::cod::COD_ARROW_UP,
        SECTION_COMMIT => crate::nerd_font::symbols::cod::COD_GIT_COMMIT,
        _ => crate::nerd_font::symbols::cod::COD_GIT_COMMIT,
    };
    format!("{icon} {}", title.as_ref())
}

fn section_with_placeholder(id: &str, title: String, items: Vec<SectionItem>) -> Option<Section> {
    if items.is_empty() {
        None
    } else {
        Some(Section::new(id, title).with_items(items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_titles_include_expected_icons() {
        let staged = git_section_title(SECTION_STAGED, "Staged changes (1)");
        let commit = git_section_title(SECTION_COMMIT, "Commit");
        assert!(staged.starts_with(crate::nerd_font::symbols::cod::COD_CHECK));
        assert!(commit.starts_with(crate::nerd_font::symbols::cod::COD_GIT_COMMIT));
    }
}
