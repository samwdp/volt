use editor_core::{Section, SectionAction, SectionItem, SectionTree};
use editor_git::{GitStatusSnapshot, StatusEntry};
use editor_plugin_api::{
    ContextHelpEntry, ContextHelpSpec, GitCommandBinding, GitFeatureSpec, GitPrefixBinding,
    GitStatusPrefix, PluginAction, PluginCommand, PluginPackage, buffer_kinds, git_actions,
    git_hooks, git_sections,
};

pub const GIT_STATUS_KIND: &str = buffer_kinds::GIT_STATUS;
pub const GIT_COMMIT_KIND: &str = buffer_kinds::GIT_COMMIT;
pub const GIT_DIFF_KIND: &str = buffer_kinds::GIT_DIFF;
pub const GIT_LOG_KIND: &str = buffer_kinds::GIT_LOG;
pub const GIT_STASH_KIND: &str = buffer_kinds::GIT_STASH;
pub const SECTION_HEADERS: &str = git_sections::HEADERS;
pub const SECTION_IN_PROGRESS: &str = git_sections::IN_PROGRESS;
pub const SECTION_STAGED: &str = git_sections::STAGED;
pub const SECTION_UNSTAGED: &str = git_sections::UNSTAGED;
pub const SECTION_UNTRACKED: &str = git_sections::UNTRACKED;
pub const SECTION_STASHES: &str = git_sections::STASHES;
pub const SECTION_UNPULLED: &str = git_sections::UNPULLED;
pub const SECTION_UNPUSHED: &str = git_sections::UNPUSHED;
pub const SECTION_REMOTE: &str = git_sections::REMOTE;
pub const SECTION_RECENT: &str = "git.status.recent";
pub const SECTION_COMMIT: &str = git_sections::COMMIT;

fn help_entry(chord: &str, action: &str, description: &str) -> ContextHelpEntry {
    ContextHelpEntry::new(chord, action, description)
}

/// Public git feature contract used by first-party and third-party code.
pub fn feature_spec() -> GitFeatureSpec {
    GitFeatureSpec {
        status_buffer_name: "*git-status*".to_owned(),
        commit_buffer_name: "*git-commit*".to_owned(),
        branch_popup_title: "Git Branches".to_owned(),
        prefix_bindings: vec![
            GitPrefixBinding::new(
                "c",
                GitStatusPrefix::Commit,
                "commit prefix",
                "Starts the commit prefix (press c again to open commit).",
            ),
            GitPrefixBinding::new(
                "P",
                GitStatusPrefix::Push,
                "push prefix",
                "Starts the push prefix (p pushremote, u upstream).",
            ),
            GitPrefixBinding::new(
                "f",
                GitStatusPrefix::Fetch,
                "fetch prefix",
                "Starts the fetch prefix (p pushremote, u upstream, a all).",
            ),
            GitPrefixBinding::new(
                "F",
                GitStatusPrefix::Pull,
                "pull prefix",
                "Starts the pull prefix (u upstream).",
            ),
            GitPrefixBinding::new(
                "b",
                GitStatusPrefix::Branch,
                "branch prefix",
                "Starts the branch prefix (press b again for branches).",
            ),
            GitPrefixBinding::new(
                "d",
                GitStatusPrefix::Diff,
                "diff prefix",
                "Starts the diff prefix.",
            ),
            GitPrefixBinding::new(
                "l",
                GitStatusPrefix::Log,
                "log prefix",
                "Starts the log prefix.",
            ),
            GitPrefixBinding::new(
                "z",
                GitStatusPrefix::Stash,
                "stash prefix",
                "Starts the stash prefix.",
            ),
            GitPrefixBinding::new(
                "m",
                GitStatusPrefix::Merge,
                "merge prefix",
                "Starts the merge prefix.",
            ),
            GitPrefixBinding::new(
                "r",
                GitStatusPrefix::Rebase,
                "rebase prefix",
                "Starts the rebase prefix.",
            ),
            GitPrefixBinding::new(
                "A",
                GitStatusPrefix::CherryPick,
                "cherry-pick prefix",
                "Starts the cherry-pick prefix.",
            ),
            GitPrefixBinding::new(
                "V",
                GitStatusPrefix::Revert,
                "revert prefix",
                "Starts the revert prefix (V/v/s/a).",
            ),
            GitPrefixBinding::new(
                "X",
                GitStatusPrefix::Reset,
                "reset prefix",
                "Starts the reset prefix (m/s/h/k).",
            ),
        ],
        command_bindings: vec![
            GitCommandBinding::new(
                Some(GitStatusPrefix::Commit),
                "c",
                "git.status.commit",
                "open commit buffer",
                "Opens the git commit buffer.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Push),
                "p",
                "git.status.push-pushremote",
                "push to pushremote",
                "Pushes to the push remote.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Push),
                "u",
                "git.status.push-upstream",
                "push to upstream",
                "Pushes to the upstream remote.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Fetch),
                "p",
                "git.status.fetch-pushremote",
                "fetch pushremote",
                "Fetches from the push remote.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Fetch),
                "u",
                "git.status.fetch-upstream",
                "fetch upstream",
                "Fetches from the upstream remote.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Fetch),
                "a",
                "git.status.fetch-all",
                "fetch all",
                "Fetches from all remotes.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Pull),
                "u",
                "git.status.pull-upstream",
                "pull upstream",
                "Pulls from the upstream remote.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Branch),
                "b",
                "git.status.branches",
                "open branch picker",
                "Opens the git branch picker.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Branch),
                "w",
                "git.worktree.create",
                "create worktree",
                "Creates a git worktree.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Merge),
                "m",
                "git.status.merge",
                "merge",
                "Merges the selected branch.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Merge),
                "e",
                "git.status.merge-edit",
                "merge edit",
                "Merges and edits the commit message.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Merge),
                "n",
                "git.status.merge-no-commit",
                "merge no-commit",
                "Merges without committing.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Merge),
                "s",
                "git.status.merge-squash",
                "merge squash",
                "Squash merges the selected branch.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Merge),
                "p",
                "git.status.merge-preview",
                "merge preview",
                "Previews merge result.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Merge),
                "a",
                "git.status.merge-abort",
                "merge abort",
                "Aborts merge in progress.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "p",
                "git.status.rebase-pushremote",
                "rebase pushremote",
                "Rebases onto push remote.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "u",
                "git.status.rebase-upstream",
                "rebase upstream",
                "Rebases onto upstream.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "e",
                "git.status.rebase-onto",
                "rebase onto",
                "Rebases onto selected commit.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "i",
                "git.status.rebase-interactive",
                "rebase interactive",
                "Starts interactive rebase.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "r",
                "git.status.rebase-continue",
                "rebase continue",
                "Continues current rebase.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "s",
                "git.status.rebase-skip",
                "rebase skip",
                "Skips current rebase commit.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "a",
                "git.status.rebase-abort",
                "rebase abort",
                "Aborts current rebase.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "f",
                "git.status.rebase-autosquash",
                "rebase autosquash",
                "Runs autosquash rebase.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "m",
                "git.status.rebase-edit-commit",
                "rebase edit commit",
                "Edits selected commit during rebase.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "w",
                "git.status.rebase-reword",
                "rebase reword",
                "Rewords selected commit.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Rebase),
                "k",
                "git.status.rebase-remove-commit",
                "rebase drop commit",
                "Removes selected commit.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Diff),
                "d",
                "git.status.diff-dwim",
                "diff dwim",
                "Shows the most relevant diff for point.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Diff),
                "s",
                "git.status.diff-staged",
                "diff staged",
                "Shows staged diff.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Diff),
                "u",
                "git.status.diff-unstaged",
                "diff unstaged",
                "Shows unstaged diff.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Diff),
                "w",
                "git.diff",
                "open diff buffer",
                "Opens the git diff buffer.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Diff),
                "c",
                "git.status.diff-commit",
                "diff commit",
                "Diffs selected commit.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Diff),
                "t",
                "git.status.diff-stash",
                "diff stash",
                "Diffs selected stash.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Diff),
                "r",
                "git.status.diff-range",
                "diff range",
                "Diffs selected range.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Diff),
                "p",
                "git.status.diff-paths",
                "diff paths",
                "Diffs selected paths.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Log),
                "l",
                "git.log",
                "open log buffer",
                "Opens the git log buffer.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Log),
                "h",
                "git.status.log-head",
                "log head",
                "Shows head history.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Log),
                "u",
                "git.status.log-related",
                "log related",
                "Shows related history.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Log),
                "o",
                "git.status.log-other",
                "log other",
                "Shows other-side history.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Log),
                "L",
                "git.status.log-branches",
                "log branches",
                "Shows branch comparison history.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Log),
                "b",
                "git.status.log-all-branches",
                "log all branches",
                "Shows all branches history.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Log),
                "a",
                "git.status.log-all",
                "log all",
                "Shows complete history.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "z",
                "git.status.stash-both",
                "stash both",
                "Stashes index and worktree.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "i",
                "git.status.stash-index",
                "stash index",
                "Stashes index only.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "w",
                "git.status.stash-worktree",
                "stash worktree",
                "Stashes worktree only.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "x",
                "git.status.stash-keep-index",
                "stash keep index",
                "Stashes while keeping index.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "a",
                "git.status.stash-apply",
                "stash apply",
                "Applies selected stash.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "p",
                "git.status.stash-pop",
                "stash pop",
                "Pops selected stash.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "k",
                "git.status.stash-drop",
                "stash drop",
                "Drops selected stash.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "v",
                "git.status.stash-show",
                "stash show",
                "Shows selected stash.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Stash),
                "l",
                "git.stash-list",
                "open stash list",
                "Opens the git stash list buffer.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::CherryPick),
                "A",
                "git.status.cherry-pick",
                "cherry-pick",
                "Cherry-picks selected commit.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::CherryPick),
                "a",
                "git.status.cherry-pick-apply",
                "apply commit at point",
                "Applies the commit under the cursor.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::CherryPick),
                "s",
                "git.status.cherry-pick-skip",
                "cherry-pick skip",
                "Skips current cherry-pick.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Revert),
                "V",
                "git.status.revert",
                "revert / continue",
                "Reverts the selected commit or continues.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Revert),
                "v",
                "git.status.revert-no-commit",
                "revert no-commit / abort",
                "Reverts without commit or aborts in progress.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Revert),
                "s",
                "git.status.revert-skip",
                "revert skip",
                "Skips the current revert/cherry-pick.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Revert),
                "a",
                "git.status.revert-abort",
                "revert abort",
                "Aborts the current revert/cherry-pick.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Reset),
                "m",
                "git.status.reset-mixed",
                "reset mixed",
                "Resets to the selected commit (mixed).",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Reset),
                "s",
                "git.status.reset-soft",
                "reset soft",
                "Resets to the selected commit (soft).",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Reset),
                "h",
                "git.status.reset-hard",
                "reset hard",
                "Resets to the selected commit (hard).",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Reset),
                "k",
                "git.status.reset-keep",
                "reset keep",
                "Resets to the selected commit (keep).",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Reset),
                "i",
                "git.status.reset-index",
                "reset index",
                "Resets index only.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Reset),
                "w",
                "git.status.reset-worktree",
                "reset worktree",
                "Resets worktree only.",
            ),
            GitCommandBinding::new(
                Some(GitStatusPrefix::Reset),
                "f",
                "git.status.checkout-file",
                "checkout file",
                "Restores file at point.",
            ),
            GitCommandBinding::new(
                None,
                "g",
                "git.status.refresh",
                "refresh status",
                "Refreshes the git status buffer.",
            ),
            GitCommandBinding::new(
                None,
                "n",
                "git.status.next-section",
                "next section",
                "Moves to the next git status section.",
            ),
            GitCommandBinding::new(
                None,
                "p",
                "git.status.previous-section",
                "previous section",
                "Moves to the previous git status section.",
            ),
            GitCommandBinding::new(
                None,
                "S",
                "git.status.stage-all",
                "stage all",
                "Stages all unstaged changes.",
            ),
            GitCommandBinding::new(
                None,
                "s",
                "git.status.stage",
                "stage file / stage all",
                "Stages the file under the cursor, or all if none is selected.",
            ),
            GitCommandBinding::new(
                None,
                "u",
                "git.status.unstage",
                "unstage file",
                "Unstages the file under the cursor.",
            ),
            GitCommandBinding::new(
                None,
                "U",
                "git.status.unstage-all",
                "unstage all",
                "Unstages all staged changes.",
            ),
            GitCommandBinding::new(
                None,
                "Y",
                "git.status.cherry-open",
                "cherry-pick prefix",
                "Opens cherry-pick targets.",
            ),
            GitCommandBinding::new(
                None,
                "a",
                "git.status.apply-commit",
                "cherry-pick apply at point",
                "Applies the commit under the cursor.",
            ),
            GitCommandBinding::new(
                None,
                "x",
                "git.status.discard-or-reset",
                "delete file(s)",
                "Deletes the file under the cursor or the visual selection.",
            ),
            GitCommandBinding::new(
                None,
                "q",
                "buffer.close",
                "close",
                "Closes the active git buffer.",
            ),
        ],
        status_help: ContextHelpSpec::new(
            "GitStatus",
            "Git Status",
            vec![
                help_entry("g", "refresh status", "Refreshes the git status buffer."),
                help_entry("n", "next section", "Moves to the next git status section."),
                help_entry(
                    "p",
                    "previous section",
                    "Moves to the previous git status section.",
                ),
                help_entry(
                    "s",
                    "stage file / stage all",
                    "Stages the file under the cursor, or all if none is selected.",
                ),
                help_entry("S", "stage all", "Stages all unstaged changes."),
                help_entry("u", "unstage file", "Unstages the file under the cursor."),
                help_entry("U", "unstage all", "Unstages all staged changes."),
                help_entry(
                    "c",
                    "commit prefix",
                    "Starts the commit prefix (press c again to open commit).",
                ),
                help_entry("c c", "open commit buffer", "Opens the git commit buffer."),
                help_entry(
                    "P",
                    "push prefix",
                    "Starts the push prefix (p pushremote, u upstream).",
                ),
                help_entry("P p", "push to pushremote", "Pushes to the push remote."),
                help_entry("P u", "push to upstream", "Pushes to the upstream remote."),
                help_entry(
                    "f",
                    "fetch prefix",
                    "Starts the fetch prefix (p pushremote, u upstream, a all).",
                ),
                help_entry("f p", "fetch pushremote", "Fetches from the push remote."),
                help_entry("f u", "fetch upstream", "Fetches from the upstream remote."),
                help_entry("f a", "fetch all", "Fetches from all remotes."),
                help_entry("F", "pull prefix", "Starts the pull prefix (u upstream)."),
                help_entry("F u", "pull upstream", "Pulls from the upstream remote."),
                help_entry(
                    "b",
                    "branch prefix",
                    "Starts the branch prefix (press b again for branches).",
                ),
                help_entry("b b", "open branch picker", "Opens the git branch picker."),
                help_entry("b w", "create worktree", "Creates a git worktree."),
                help_entry("d", "diff prefix", "Starts the diff prefix."),
                help_entry(
                    "d d",
                    "diff dwim",
                    "Shows the most relevant diff for point.",
                ),
                help_entry("d s", "diff staged", "Shows staged diff."),
                help_entry("d u", "diff unstaged", "Shows unstaged diff."),
                help_entry("d w", "open diff buffer", "Opens the git diff buffer."),
                help_entry("d c", "diff commit", "Diffs selected commit."),
                help_entry("d t", "diff stash", "Diffs selected stash."),
                help_entry("d r", "diff range", "Diffs selected range."),
                help_entry("d p", "diff paths", "Diffs selected paths."),
                help_entry("l", "log prefix", "Starts the log prefix."),
                help_entry("l l", "open log buffer", "Opens the git log buffer."),
                help_entry("l h", "log head", "Shows head history."),
                help_entry("l u", "log related", "Shows related history."),
                help_entry("l o", "log other", "Shows other-side history."),
                help_entry("l L", "log branches", "Shows branch comparison history."),
                help_entry("l b", "log all branches", "Shows all branches history."),
                help_entry("l a", "log all", "Shows complete history."),
                help_entry("z", "stash prefix", "Starts the stash prefix."),
                help_entry("z z", "stash both", "Stashes index and worktree."),
                help_entry("z i", "stash index", "Stashes index only."),
                help_entry("z w", "stash worktree", "Stashes worktree only."),
                help_entry("z x", "stash keep index", "Stashes while keeping index."),
                help_entry("z a", "stash apply", "Applies selected stash."),
                help_entry("z p", "stash pop", "Pops selected stash."),
                help_entry("z k", "stash drop", "Drops selected stash."),
                help_entry("z v", "stash show", "Shows selected stash."),
                help_entry("z l", "open stash list", "Opens the git stash list buffer."),
                help_entry("m", "merge prefix", "Starts the merge prefix."),
                help_entry("m m", "merge", "Merges the selected branch."),
                help_entry("m e", "merge edit", "Merges and edits the commit message."),
                help_entry("m n", "merge no-commit", "Merges without committing."),
                help_entry("m s", "merge squash", "Squash merges the selected branch."),
                help_entry("m p", "merge preview", "Previews merge result."),
                help_entry("m a", "merge abort", "Aborts merge in progress."),
                help_entry("r", "rebase prefix", "Starts the rebase prefix."),
                help_entry("r p", "rebase pushremote", "Rebases onto push remote."),
                help_entry("r u", "rebase upstream", "Rebases onto upstream."),
                help_entry("r e", "rebase onto", "Rebases onto selected commit."),
                help_entry("r i", "rebase interactive", "Starts interactive rebase."),
                help_entry("r r", "rebase continue", "Continues current rebase."),
                help_entry("r s", "rebase skip", "Skips current rebase commit."),
                help_entry("r a", "rebase abort", "Aborts current rebase."),
                help_entry("r f", "rebase autosquash", "Runs autosquash rebase."),
                help_entry(
                    "r m",
                    "rebase edit commit",
                    "Edits selected commit during rebase.",
                ),
                help_entry("r w", "rebase reword", "Rewords selected commit."),
                help_entry("r k", "rebase drop commit", "Removes selected commit."),
                help_entry("A", "cherry-pick prefix", "Starts the cherry-pick prefix."),
                help_entry("A A", "cherry-pick", "Cherry-picks selected commit."),
                help_entry(
                    "A a",
                    "apply commit at point",
                    "Applies the commit under the cursor.",
                ),
                help_entry("A s", "cherry-pick skip", "Skips current cherry-pick."),
                help_entry("V", "revert prefix", "Starts the revert prefix (V/v/s/a)."),
                help_entry(
                    "V V",
                    "revert / continue",
                    "Reverts the selected commit or continues.",
                ),
                help_entry(
                    "V v",
                    "revert no-commit / abort",
                    "Reverts without commit or aborts in progress.",
                ),
                help_entry(
                    "V s",
                    "revert skip",
                    "Skips the current revert/cherry-pick.",
                ),
                help_entry(
                    "V a",
                    "revert abort",
                    "Aborts the current revert/cherry-pick.",
                ),
                help_entry("X", "reset prefix", "Starts the reset prefix (m/s/h/k)."),
                help_entry(
                    "X m",
                    "reset mixed",
                    "Resets to the selected commit (mixed).",
                ),
                help_entry("X s", "reset soft", "Resets to the selected commit (soft)."),
                help_entry("X h", "reset hard", "Resets to the selected commit (hard)."),
                help_entry("X k", "reset keep", "Resets to the selected commit (keep)."),
                help_entry("X i", "reset index", "Resets index only."),
                help_entry("X w", "reset worktree", "Resets worktree only."),
                help_entry("X f", "checkout file", "Restores file at point."),
                help_entry("q", "close", "Closes the active git buffer."),
            ],
        ),
        view_help: ContextHelpSpec::new(
            "GitView",
            "Git View",
            vec![help_entry(
                "g",
                "refresh view",
                "Refreshes git diff/log/stash buffers.",
            )],
        ),
    }
}

/// Returns the git status prefix started by a chord, if any.
pub fn status_prefix_for_chord(chord: &str) -> Option<GitStatusPrefix> {
    feature_spec().prefix_for_chord(chord)
}

/// Resolves a git status prefix + chord pair to the command it should execute.
pub fn status_command_name(prefix: Option<GitStatusPrefix>, chord: &str) -> Option<&'static str> {
    let spec = feature_spec();
    let command = spec.command_for_chord(prefix, chord)?;
    match command {
        "git.status.commit" => Some("git.status.commit"),
        "git.status.push-pushremote" => Some("git.status.push-pushremote"),
        "git.status.push-upstream" => Some("git.status.push-upstream"),
        "git.status.fetch-pushremote" => Some("git.status.fetch-pushremote"),
        "git.status.fetch-upstream" => Some("git.status.fetch-upstream"),
        "git.status.fetch-all" => Some("git.status.fetch-all"),
        "git.status.pull-upstream" => Some("git.status.pull-upstream"),
        "git.status.branches" => Some("git.status.branches"),
        "git.worktree.create" => Some("git.worktree.create"),
        "git.status.merge" => Some("git.status.merge"),
        "git.status.merge-edit" => Some("git.status.merge-edit"),
        "git.status.merge-no-commit" => Some("git.status.merge-no-commit"),
        "git.status.merge-squash" => Some("git.status.merge-squash"),
        "git.status.merge-preview" => Some("git.status.merge-preview"),
        "git.status.merge-abort" => Some("git.status.merge-abort"),
        "git.status.rebase-pushremote" => Some("git.status.rebase-pushremote"),
        "git.status.rebase-upstream" => Some("git.status.rebase-upstream"),
        "git.status.rebase-onto" => Some("git.status.rebase-onto"),
        "git.status.rebase-interactive" => Some("git.status.rebase-interactive"),
        "git.status.rebase-continue" => Some("git.status.rebase-continue"),
        "git.status.rebase-skip" => Some("git.status.rebase-skip"),
        "git.status.rebase-abort" => Some("git.status.rebase-abort"),
        "git.status.rebase-autosquash" => Some("git.status.rebase-autosquash"),
        "git.status.rebase-edit-commit" => Some("git.status.rebase-edit-commit"),
        "git.status.rebase-reword" => Some("git.status.rebase-reword"),
        "git.status.rebase-remove-commit" => Some("git.status.rebase-remove-commit"),
        "git.status.diff-dwim" => Some("git.status.diff-dwim"),
        "git.status.diff-staged" => Some("git.status.diff-staged"),
        "git.status.diff-unstaged" => Some("git.status.diff-unstaged"),
        "git.diff" => Some("git.diff"),
        "git.status.diff-commit" => Some("git.status.diff-commit"),
        "git.status.diff-stash" => Some("git.status.diff-stash"),
        "git.status.diff-range" => Some("git.status.diff-range"),
        "git.status.diff-paths" => Some("git.status.diff-paths"),
        "git.log" => Some("git.log"),
        "git.status.log-head" => Some("git.status.log-head"),
        "git.status.log-related" => Some("git.status.log-related"),
        "git.status.log-other" => Some("git.status.log-other"),
        "git.status.log-branches" => Some("git.status.log-branches"),
        "git.status.log-all-branches" => Some("git.status.log-all-branches"),
        "git.status.log-all" => Some("git.status.log-all"),
        "git.status.stash-both" => Some("git.status.stash-both"),
        "git.status.stash-index" => Some("git.status.stash-index"),
        "git.status.stash-worktree" => Some("git.status.stash-worktree"),
        "git.status.stash-keep-index" => Some("git.status.stash-keep-index"),
        "git.status.stash-apply" => Some("git.status.stash-apply"),
        "git.status.stash-pop" => Some("git.status.stash-pop"),
        "git.status.stash-drop" => Some("git.status.stash-drop"),
        "git.status.stash-show" => Some("git.status.stash-show"),
        "git.stash-list" => Some("git.stash-list"),
        "git.status.cherry-pick" => Some("git.status.cherry-pick"),
        "git.status.cherry-pick-apply" => Some("git.status.cherry-pick-apply"),
        "git.status.cherry-pick-skip" => Some("git.status.cherry-pick-skip"),
        "git.status.revert" => Some("git.status.revert"),
        "git.status.revert-no-commit" => Some("git.status.revert-no-commit"),
        "git.status.revert-skip" => Some("git.status.revert-skip"),
        "git.status.revert-abort" => Some("git.status.revert-abort"),
        "git.status.reset-mixed" => Some("git.status.reset-mixed"),
        "git.status.reset-soft" => Some("git.status.reset-soft"),
        "git.status.reset-hard" => Some("git.status.reset-hard"),
        "git.status.reset-keep" => Some("git.status.reset-keep"),
        "git.status.reset-index" => Some("git.status.reset-index"),
        "git.status.reset-worktree" => Some("git.status.reset-worktree"),
        "git.status.checkout-file" => Some("git.status.checkout-file"),
        "git.status.refresh" => Some("git.status.refresh"),
        "git.status.next-section" => Some("git.status.next-section"),
        "git.status.previous-section" => Some("git.status.previous-section"),
        "git.status.stage-all" => Some("git.status.stage-all"),
        "git.status.stage" => Some("git.status.stage"),
        "git.status.unstage" => Some("git.status.unstage"),
        "git.status.unstage-all" => Some("git.status.unstage-all"),
        "git.status.cherry-open" => Some("git.status.cherry-open"),
        "git.status.apply-commit" => Some("git.status.apply-commit"),
        "git.status.discard-or-reset" => Some("git.status.discard-or-reset"),
        "buffer.close" => Some("buffer.close"),
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
                git_hooks::STATUS_OPEN_POPUP,
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
            vec![PluginAction::emit_hook(git_hooks::DIFF_OPEN, None::<&str>)],
        ),
        PluginCommand::new(
            "git.log",
            "Opens the git log buffer.",
            vec![PluginAction::emit_hook(git_hooks::LOG_OPEN, None::<&str>)],
        ),
        PluginCommand::new(
            "git.stash-list",
            "Opens the git stash list buffer.",
            vec![PluginAction::emit_hook(
                git_hooks::STASH_LIST_OPEN,
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
    if let Some(section) = recent_section(snapshot) {
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
            let action = SectionAction::new(git_actions::UNSTAGE_FILE).with_detail(entry.path());
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
            let action = SectionAction::new(git_actions::STAGE_FILE).with_detail(entry.path());
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
            let action = SectionAction::new(git_actions::STAGE_FILE).with_detail(path);
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
            let action = SectionAction::new(git_actions::SHOW_STASH).with_detail(entry.name());
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
    let upstream = snapshot.upstream()?;
    let items = snapshot
        .unpulled()
        .iter()
        .map(|entry| {
            let action = SectionAction::new(git_actions::SHOW_COMMIT).with_detail(entry.hash());
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
        git_section_title(
            SECTION_UNPULLED,
            format!("Unpulled from {upstream} ({})", snapshot.unpulled().len()),
        ),
        items,
    )
}

fn unpushed_section(snapshot: &GitStatusSnapshot) -> Option<Section> {
    let upstream = snapshot.upstream()?;
    let items = snapshot
        .unpushed()
        .iter()
        .map(|entry| {
            let action = SectionAction::new(git_actions::SHOW_COMMIT).with_detail(entry.hash());
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
        git_section_title(
            SECTION_UNPUSHED,
            format!("Unpushed to {upstream} ({})", snapshot.unpushed().len()),
        ),
        items,
    )
}

fn recent_section(snapshot: &GitStatusSnapshot) -> Option<Section> {
    if snapshot.recent().is_empty() {
        return None;
    }
    if snapshot.upstream().is_some()
        && (!snapshot.unpulled().is_empty() || !snapshot.unpushed().is_empty())
    {
        return None;
    }
    let items = snapshot
        .recent()
        .iter()
        .map(|entry| {
            let action = SectionAction::new(git_actions::SHOW_COMMIT).with_detail(entry.hash());
            SectionItem::new(format!(
                "{} {} {}",
                crate::icon_font::symbols::cod::COD_HISTORY,
                entry.hash(),
                entry.summary()
            ))
            .with_action(action)
        })
        .collect::<Vec<_>>();
    section_with_placeholder(
        SECTION_RECENT,
        git_section_title(
            SECTION_RECENT,
            format!("Recent commits ({})", snapshot.recent().len()),
        ),
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
        .with_action(SectionAction::new(git_actions::COMMIT_OPEN))
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
        SECTION_RECENT => crate::icon_font::symbols::cod::COD_HISTORY,
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
    use editor_git::GitLogEntry;

    fn log_entry(hash: &str, summary: &str) -> GitLogEntry {
        GitLogEntry::new(hash.to_owned(), summary.to_owned())
    }

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

    #[test]
    fn status_sections_show_recent_commits_once_without_upstream() {
        let snapshot = GitStatusSnapshot::default().with_recent(vec![log_entry("abc123", "seed")]);
        let sections = status_sections(&snapshot);
        let ids = sections
            .sections()
            .iter()
            .map(|section| section.id())
            .collect::<Vec<_>>();
        assert!(ids.contains(&SECTION_RECENT));
        assert!(!ids.contains(&SECTION_UNPULLED));
        assert!(!ids.contains(&SECTION_UNPUSHED));
    }

    #[test]
    fn status_sections_show_recent_commits_when_tracking_lists_are_empty() {
        let snapshot = GitStatusSnapshot::default()
            .with_upstreams(Some("feature/TASK-123-abc".to_owned()), None)
            .with_recent(vec![log_entry("abc123", "seed")]);
        let sections = status_sections(&snapshot);
        let recent = sections
            .sections()
            .iter()
            .find(|section| section.id() == SECTION_RECENT)
            .expect("recent section should be present");
        assert_eq!(
            recent.title(),
            &git_section_title(SECTION_RECENT, "Recent commits (1)")
        );
        assert_eq!(
            recent.items()[0].text(),
            format!(
                "{} abc123 seed",
                crate::icon_font::symbols::cod::COD_HISTORY
            )
        );
    }

    #[test]
    fn status_sections_hide_recent_commits_when_tracking_lists_have_entries() {
        let snapshot = GitStatusSnapshot::default()
            .with_upstreams(Some("feature/TASK-123-abc".to_owned()), None)
            .with_recent(vec![log_entry("abc123", "seed")])
            .with_unpushed(vec![log_entry("def456", "ahead")]);
        let sections = status_sections(&snapshot);
        let ids = sections
            .sections()
            .iter()
            .map(|section| section.id())
            .collect::<Vec<_>>();
        assert!(ids.contains(&SECTION_UNPUSHED));
        assert!(!ids.contains(&SECTION_RECENT));
    }
}
