use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

const STANDALONE_USER_GITIGNORE: &str = "target/\nsdk/\n";

pub fn setup_standalone_user_repository(
    user_destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        user_destination.join(".gitignore"),
        STANDALONE_USER_GITIGNORE,
    )?;

    let status = Command::new("git")
        .arg("init")
        .arg("--quiet")
        .current_dir(user_destination)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err("git init failed for standalone user directory".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{STANDALONE_USER_GITIGNORE, setup_standalone_user_repository};
    use std::{
        env, fs,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn setup_standalone_user_repository_writes_gitignore_and_initializes_git() {
        let temp_root = env::temp_dir().join(format!(
            "volt-standalone-user-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_millis()
        ));
        fs::create_dir_all(&temp_root).expect("create temp root");

        setup_standalone_user_repository(&temp_root).expect("setup standalone user repository");

        assert_eq!(
            fs::read_to_string(temp_root.join(".gitignore")).expect("read .gitignore"),
            STANDALONE_USER_GITIGNORE
        );
        assert!(temp_root.join(".git").is_dir());

        let output = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(&temp_root)
            .output()
            .expect("run git rev-parse");
        assert!(output.status.success(), "git rev-parse should succeed");
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "true",
            "standalone user directory should be a git work tree"
        );

        fs::remove_dir_all(&temp_root).expect("cleanup temp root");
    }
}
