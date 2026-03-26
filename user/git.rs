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
pub const SECTION_REMOTE: &str = "git.status.remote";
pub const SECTION_COMMIT: &str = "git.status.commit";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatusPrefix {
    Commit,
    Push,
    Fetch,
    Pull,
    Branch,
    Diff,
    Log,
    Stash,
    Merge,
    Rebase,
    CherryPick,
    Revert,
    Reset,
}

/// Returns the git status prefix started by a chord, if any.
pub fn status_prefix_for_chord(chord: &str) -> Option<GitStatusPrefix> {
    match chord {
        "c" => Some(GitStatusPrefix::Commit),
        "P" => Some(GitStatusPrefix::Push),
        "f" => Some(GitStatusPrefix::Fetch),
        "F" => Some(GitStatusPrefix::Pull),
        "b" => Some(GitStatusPrefix::Branch),
        "d" => Some(GitStatusPrefix::Diff),
        "l" => Some(GitStatusPrefix::Log),
        "z" => Some(GitStatusPrefix::Stash),
        "m" => Some(GitStatusPrefix::Merge),
        "r" => Some(GitStatusPrefix::Rebase),
        "A" => Some(GitStatusPrefix::CherryPick),
        "V" => Some(GitStatusPrefix::Revert),
        "X" => Some(GitStatusPrefix::Reset),
        _ => None,
    }
}

/// Resolves a git status prefix + chord pair to the command it should execute.
pub fn status_command_name(prefix: Option<GitStatusPrefix>, chord: &str) -> Option<&'static str> {
    match (prefix, chord) {
        (Some(GitStatusPrefix::Commit), "c") => Some("git.status.commit"),
        (Some(GitStatusPrefix::Push), "p") => Some("git.status.push-pushremote"),
        (Some(GitStatusPrefix::Push), "u") => Some("git.status.push-upstream"),
        (Some(GitStatusPrefix::Fetch), "p") => Some("git.status.fetch-pushremote"),
        (Some(GitStatusPrefix::Fetch), "u") => Some("git.status.fetch-upstream"),
        (Some(GitStatusPrefix::Fetch), "a") => Some("git.status.fetch-all"),
        (Some(GitStatusPrefix::Pull), "u") => Some("git.status.pull-upstream"),
        (Some(GitStatusPrefix::Branch), "b") => Some("git.status.branches"),
        (Some(GitStatusPrefix::Merge), "m") => Some("git.status.merge"),
        (Some(GitStatusPrefix::Merge), "e") => Some("git.status.merge-edit"),
        (Some(GitStatusPrefix::Merge), "n") => Some("git.status.merge-no-commit"),
        (Some(GitStatusPrefix::Merge), "s") => Some("git.status.merge-squash"),
        (Some(GitStatusPrefix::Merge), "p") => Some("git.status.merge-preview"),
        (Some(GitStatusPrefix::Merge), "a") => Some("git.status.merge-abort"),
        (Some(GitStatusPrefix::Rebase), "p") => Some("git.status.rebase-pushremote"),
        (Some(GitStatusPrefix::Rebase), "u") => Some("git.status.rebase-upstream"),
        (Some(GitStatusPrefix::Rebase), "e") => Some("git.status.rebase-onto"),
        (Some(GitStatusPrefix::Rebase), "i") => Some("git.status.rebase-interactive"),
        (Some(GitStatusPrefix::Rebase), "r") => Some("git.status.rebase-continue"),
        (Some(GitStatusPrefix::Rebase), "s") => Some("git.status.rebase-skip"),
        (Some(GitStatusPrefix::Rebase), "a") => Some("git.status.rebase-abort"),
        (Some(GitStatusPrefix::Rebase), "f") => Some("git.status.rebase-autosquash"),
        (Some(GitStatusPrefix::Rebase), "m") => Some("git.status.rebase-edit-commit"),
        (Some(GitStatusPrefix::Rebase), "w") => Some("git.status.rebase-reword"),
        (Some(GitStatusPrefix::Rebase), "k") => Some("git.status.rebase-remove-commit"),
        (Some(GitStatusPrefix::Diff), "d") => Some("git.status.diff-dwim"),
        (Some(GitStatusPrefix::Diff), "s") => Some("git.status.diff-staged"),
        (Some(GitStatusPrefix::Diff), "u") => Some("git.status.diff-unstaged"),
        (Some(GitStatusPrefix::Diff), "w") => Some("git.diff"),
        (Some(GitStatusPrefix::Diff), "c") => Some("git.status.diff-commit"),
        (Some(GitStatusPrefix::Diff), "t") => Some("git.status.diff-stash"),
        (Some(GitStatusPrefix::Diff), "r") => Some("git.status.diff-range"),
        (Some(GitStatusPrefix::Diff), "p") => Some("git.status.diff-paths"),
        (Some(GitStatusPrefix::Log), "l") => Some("git.log"),
        (Some(GitStatusPrefix::Log), "h") => Some("git.status.log-head"),
        (Some(GitStatusPrefix::Log), "u") => Some("git.status.log-related"),
        (Some(GitStatusPrefix::Log), "o") => Some("git.status.log-other"),
        (Some(GitStatusPrefix::Log), "L") => Some("git.status.log-branches"),
        (Some(GitStatusPrefix::Log), "b") => Some("git.status.log-all-branches"),
        (Some(GitStatusPrefix::Log), "a") => Some("git.status.log-all"),
        (Some(GitStatusPrefix::Stash), "z") => Some("git.status.stash-both"),
        (Some(GitStatusPrefix::Stash), "i") => Some("git.status.stash-index"),
        (Some(GitStatusPrefix::Stash), "w") => Some("git.status.stash-worktree"),
        (Some(GitStatusPrefix::Stash), "x") => Some("git.status.stash-keep-index"),
        (Some(GitStatusPrefix::Stash), "a") => Some("git.status.stash-apply"),
        (Some(GitStatusPrefix::Stash), "p") => Some("git.status.stash-pop"),
        (Some(GitStatusPrefix::Stash), "k") => Some("git.status.stash-drop"),
        (Some(GitStatusPrefix::Stash), "v") => Some("git.status.stash-show"),
        (Some(GitStatusPrefix::Stash), "l") => Some("git.stash-list"),
        (Some(GitStatusPrefix::CherryPick), "A") => Some("git.status.cherry-pick"),
        (Some(GitStatusPrefix::CherryPick), "a") => Some("git.status.cherry-pick-apply"),
        (Some(GitStatusPrefix::CherryPick), "s") => Some("git.status.cherry-pick-skip"),
        (Some(GitStatusPrefix::Revert), "V") => Some("git.status.revert"),
        (Some(GitStatusPrefix::Revert), "v") => Some("git.status.revert-no-commit"),
        (Some(GitStatusPrefix::Revert), "s") => Some("git.status.revert-skip"),
        (Some(GitStatusPrefix::Revert), "a") => Some("git.status.revert-abort"),
        (Some(GitStatusPrefix::Reset), "m") => Some("git.status.reset-mixed"),
        (Some(GitStatusPrefix::Reset), "s") => Some("git.status.reset-soft"),
        (Some(GitStatusPrefix::Reset), "h") => Some("git.status.reset-hard"),
        (Some(GitStatusPrefix::Reset), "k") => Some("git.status.reset-keep"),
        (Some(GitStatusPrefix::Reset), "i") => Some("git.status.reset-index"),
        (Some(GitStatusPrefix::Reset), "w") => Some("git.status.reset-worktree"),
        (Some(GitStatusPrefix::Reset), "f") => Some("git.status.checkout-file"),
        (None, "g") => Some("git.status.refresh"),
        (None, "n") => Some("git.status.next-section"),
        (None, "p") => Some("git.status.previous-section"),
        (None, "S") => Some("git.status.stage-all"),
        (None, "s") => Some("git.status.stage"),
        (None, "u") => Some("git.status.unstage"),
        (None, "U") => Some("git.status.unstage-all"),
        (None, "Y") => Some("git.status.cherry-open"),
        (None, "a") => Some("git.status.apply-commit"),
        (None, "x") => Some("git.status.discard-or-reset"),
        (None, "q") => Some("buffer.close"),
        _ => None,
    }
}

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
    if let Some(section) = remote_section(snapshot) {
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
    let branch_icon = crate::icon_font::symbols::dev::DEV_GIT_BRANCH;
    let incoming_icon = crate::icon_font::symbols::cod::COD_ARROW_DOWN;
    let outgoing_icon = crate::icon_font::symbols::cod::COD_ARROW_UP;
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
                crate::icon_font::symbols::cod::COD_LOADING
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
                "{} {path}",
                crate::icon_font::symbols::cod::COD_SYMBOL_FILE
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
            let name = stash_display_name(entry.name());
            SectionItem::new(format!(
                "{} {} {}",
                crate::icon_font::symbols::cod::COD_HISTORY,
                name,
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
                crate::icon_font::symbols::cod::COD_ARROW_DOWN,
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
                crate::icon_font::symbols::cod::COD_ARROW_UP,
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
            crate::icon_font::symbols::cod::COD_GIT_COMMIT
        ))
    } else {
        SectionItem::new(format!(
            "{} Press c to commit staged changes.",
            crate::icon_font::symbols::cod::COD_GIT_COMMIT
        ))
        .with_action(SectionAction::new(ACTION_COMMIT_OPEN))
    };
    Section::new(SECTION_COMMIT, git_section_title(SECTION_COMMIT, "Commit")).with_items(vec![item])
}

fn remote_section(snapshot: &GitStatusSnapshot) -> Option<Section> {
    let mut items = Vec::new();
    if let Some(upstream) = snapshot.upstream() {
        items.push(SectionItem::new(format!(
            "{} Press F u to pull from {upstream}.",
            crate::icon_font::symbols::cod::COD_ARROW_DOWN
        )));
    }
    section_with_placeholder(
        SECTION_REMOTE,
        git_section_title(SECTION_REMOTE, "Remote"),
        items,
    )
}

fn status_entry_label(entry: &StatusEntry, staged: bool) -> String {
    let code = if staged {
        entry.index_status()
    } else {
        entry.worktree_status()
    };
    let icon = status_entry_icon(code);
    format!("{icon} {}", entry.path())
}

fn status_entry_icon(code: char) -> &'static str {
    match code {
        'A' => crate::icon_font::symbols::cod::COD_DIFF_ADDED,
        'D' => crate::icon_font::symbols::cod::COD_DIFF_REMOVED,
        'M' => crate::icon_font::symbols::cod::COD_DIFF_MODIFIED,
        'R' => crate::icon_font::symbols::cod::COD_DIFF_RENAMED,
        'C' => crate::icon_font::symbols::cod::COD_ARROW_SWAP,
        'U' => crate::icon_font::symbols::cod::COD_SYNC,
        _ => crate::icon_font::symbols::cod::COD_DIFF_MODIFIED,
    }
}

fn stash_display_name(name: &str) -> String {
    if let Some(index) = name
        .strip_prefix("stash@{")
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return format!("stash[{index}]");
    }
    name.to_owned()
}

fn git_section_title(id: &str, title: impl AsRef<str>) -> String {
    let icon = match id {
        SECTION_HEADERS => crate::icon_font::symbols::dev::DEV_GIT_BRANCH,
        SECTION_IN_PROGRESS => crate::icon_font::symbols::cod::COD_LOADING,
        SECTION_STAGED => crate::icon_font::symbols::cod::COD_CHECK,
        SECTION_UNSTAGED => crate::icon_font::symbols::cod::COD_DIFF_MODIFIED,
        SECTION_UNTRACKED => crate::icon_font::symbols::cod::COD_SYMBOL_FILE,
        SECTION_STASHES => crate::icon_font::symbols::cod::COD_HISTORY,
        SECTION_UNPULLED => crate::icon_font::symbols::cod::COD_ARROW_DOWN,
        SECTION_UNPUSHED => crate::icon_font::symbols::cod::COD_ARROW_UP,
        SECTION_REMOTE => crate::icon_font::symbols::cod::COD_ARROW_DOWN,
        SECTION_COMMIT => crate::icon_font::symbols::cod::COD_GIT_COMMIT,
        _ => crate::icon_font::symbols::cod::COD_GIT_COMMIT,
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
    fn git_status_keymaps_export_prefix_starters_and_commands() {
        assert_eq!(status_prefix_for_chord("F"), Some(GitStatusPrefix::Pull));
        assert_eq!(status_prefix_for_chord("X"), Some(GitStatusPrefix::Reset));
        assert_eq!(status_prefix_for_chord("q"), None);
        assert_eq!(status_command_name(None, "S"), Some("git.status.stage-all"));
        assert_eq!(
            status_command_name(Some(GitStatusPrefix::Pull), "u"),
            Some("git.status.pull-upstream")
        );
        assert_eq!(
            status_command_name(Some(GitStatusPrefix::Diff), "w"),
            Some("git.diff")
        );
    }

    #[test]
    fn section_titles_include_expected_icons() {
        let staged = git_section_title(SECTION_STAGED, "Staged changes (1)");
        let remote = git_section_title(SECTION_REMOTE, "Remote");
        let commit = git_section_title(SECTION_COMMIT, "Commit");
        assert!(staged.starts_with(crate::icon_font::symbols::cod::COD_CHECK));
        assert!(remote.starts_with(crate::icon_font::symbols::cod::COD_ARROW_DOWN));
        assert!(commit.starts_with(crate::icon_font::symbols::cod::COD_GIT_COMMIT));
    }

    #[test]
    fn status_entries_and_untracked_items_omit_status_words() {
        let status =
            editor_git::parse_status(" M src/main.rs\n?? notes.txt\n").expect("status snapshot");
        assert_eq!(
            status_entry_label(&status.unstaged()[0], false),
            format!(
                "{} src/main.rs",
                crate::icon_font::symbols::cod::COD_DIFF_MODIFIED
            )
        );

        let snapshot = GitStatusSnapshot::default().with_status(status);
        let section = untracked_section(&snapshot).expect("untracked section");
        assert_eq!(
            section.items()[0].text(),
            format!(
                "{} notes.txt",
                crate::icon_font::symbols::cod::COD_SYMBOL_FILE
            )
        );
    }

    #[test]
    fn stashes_display_compact_indices() {
        let snapshot = GitStatusSnapshot::default().with_stashes(editor_git::parse_stash_list(
            "stash@{0}: WIP on master: overnight todo",
        ));
        let section = stashes_section(&snapshot);
        assert_eq!(
            section.items()[0].text(),
            format!(
                "{} stash[0] WIP on master: overnight todo",
                crate::icon_font::symbols::cod::COD_HISTORY
            )
        );
    }

    #[test]
    fn status_sections_include_pull_command_when_upstream_exists() {
        let snapshot = GitStatusSnapshot::default().with_upstreams(
            Some("origin/main".to_owned()),
            Some("origin/main".to_owned()),
        );
        let sections = status_sections(&snapshot);
        let remote = sections
            .sections()
            .iter()
            .find(|section| section.id() == SECTION_REMOTE)
            .expect("remote section should be present when upstream exists");
        assert_eq!(
            remote.items()[0].text(),
            format!(
                "{} Press F u to pull from origin/main.",
                crate::icon_font::symbols::cod::COD_ARROW_DOWN
            )
        );
    }
}
