//! End-to-end tests for the shell integration: real bash/zsh processes load
//! `git-clown init <shell>`, clone through the wrapper and must end up inside
//! the cloned repository. Shells that are not installed are skipped.

use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_git-clown");

/// A sandbox with a source repo reachable as `https://example.test/org/repo.git`
/// (through git's `insteadOf`), an empty workspace root and a fake `$HOME`.
struct Sandbox {
    dir: TempDir,
}

impl Sandbox {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let sandbox = Sandbox { dir };
        let source = sandbox.root().join("src/org/repo.git");
        fs::create_dir_all(&source).unwrap();
        let git = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(&source)
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?} failed");
        };
        git(&["init", "-q", "-b", "main"]);
        git(&[
            "-c",
            "user.name=T",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "init",
        ]);
        fs::create_dir_all(sandbox.home()).unwrap();
        sandbox
    }

    fn root(&self) -> PathBuf {
        fs::canonicalize(self.dir.path()).unwrap()
    }

    fn home(&self) -> PathBuf {
        self.root().join("home")
    }

    fn workspace(&self) -> PathBuf {
        self.root().join("ws")
    }

    /// `$PATH` with the git-clown binary's directory first.
    fn path(&self) -> OsString {
        let bin_dir = Path::new(BIN).parent().unwrap().to_path_buf();
        let current = env::var_os("PATH").unwrap_or_default();
        env::join_paths(std::iter::once(bin_dir).chain(env::split_paths(&current))).unwrap()
    }

    /// A command for `program` with the sandbox environment applied.
    fn command(&self, program: &str) -> Command {
        let mut cmd = Command::new(program);
        cmd.current_dir(self.root())
            .env("HOME", self.home())
            .env("PATH", self.path())
            .env("WORKSPACE_ROOT", self.workspace())
            .env("GIT_CONFIG_COUNT", "1")
            .env(
                "GIT_CONFIG_KEY_0",
                format!("url.file://{}/src/.insteadOf", self.root().display()),
            )
            .env("GIT_CONFIG_VALUE_0", "https://example.test/")
            .env_remove("ZDOTDIR")
            .env_remove("BASH_ENV")
            .env_remove("GIT_CLOWN_CD_FILE")
            .env_remove("GIT_CLOWN_SHELL");
        cmd
    }

    /// Runs `body` in `shell` (without rc files) after loading the integration.
    fn run_in_shell(&self, shell: &str, body: &str) -> Output {
        let script = format!("eval \"$(command git-clown init {shell})\"\n{body}");
        let no_rc: &[&str] = if shell == "zsh" {
            &["-f"]
        } else {
            &["--norc", "--noprofile"]
        };
        self.command(shell)
            .args(no_rc)
            .arg("-c")
            .arg(script)
            .output()
            .unwrap()
    }
}

fn shell_available(shell: &str) -> bool {
    Command::new(shell)
        .arg("-c")
        .arg("true")
        .output()
        .is_ok_and(|o| o.status.success())
}

fn stdout_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn clone_through_wrapper_changes_directory() {
    for shell in ["bash", "zsh"].into_iter().filter(|s| shell_available(s)) {
        let sb = Sandbox::new();
        let repo = sb.workspace().join("example.test/org/repo");

        // First clone, then a second run on the existing clone: both cd.
        let output = sb.run_in_shell(
            shell,
            "git clown https://example.test/org/repo.git || exit 1\npwd\n\
             cd / && git-clown https://example.test/org/repo.git || exit 1\npwd\n\
             git rev-parse --is-inside-work-tree",
        );
        assert_success(&output);
        let lines = stdout_lines(&output);
        let pwds: Vec<&String> = lines.iter().filter(|l| l.starts_with('/')).collect();
        assert_eq!(
            pwds,
            vec![&repo.display().to_string(); 2],
            "{shell}: {lines:?}"
        );
        // Non-clown git commands still reach the real git.
        assert_eq!(lines.last().map(String::as_str), Some("true"), "{shell}");
    }
}

#[test]
fn bare_clone_through_wrapper_changes_into_worktree() {
    for shell in ["bash", "zsh"].into_iter().filter(|s| shell_available(s)) {
        let sb = Sandbox::new();
        let output = sb.run_in_shell(
            shell,
            "git clown --bare https://example.test/org/repo.git >/dev/null || exit 1\npwd",
        );
        assert_success(&output);
        let worktree = sb.workspace().join("example.test/org/repo/main");
        assert_eq!(
            stdout_lines(&output).last(),
            Some(&worktree.display().to_string()),
            "{shell}"
        );
    }
}

#[test]
fn failed_clone_keeps_directory_and_exit_code() {
    for shell in ["bash", "zsh"].into_iter().filter(|s| shell_available(s)) {
        let sb = Sandbox::new();
        let output = sb.run_in_shell(
            shell,
            "git clown https://example.test/org/missing.git >/dev/null 2>&1\necho \"status=$?\"\npwd",
        );
        let lines = stdout_lines(&output);
        assert_ne!(lines[0], "status=0", "{shell}");
        assert_eq!(lines[1], sb.root().display().to_string(), "{shell}");
    }
}

#[test]
fn clone_without_wrapper_still_works() {
    let sb = Sandbox::new();
    let output = sb
        .command(BIN)
        .arg("https://example.test/org/repo.git")
        .output()
        .unwrap();
    assert_success(&output);
    assert!(sb.workspace().join("example.test/org/repo/.git").is_dir());
}

#[test]
fn doctor_install_is_idempotent_and_loads_in_interactive_shell() {
    for shell in ["bash", "zsh"].into_iter().filter(|s| shell_available(s)) {
        let sb = Sandbox::new();
        let rc = sb.home().join(if shell == "zsh" {
            ".zshrc"
        } else if cfg!(target_os = "macos") {
            ".bash_profile"
        } else {
            ".bashrc"
        });
        fs::write(&rc, "# existing config\n").unwrap();

        let doctor = |args: &[&str]| {
            sb.command(BIN)
                .arg("doctor")
                .args(args)
                .args(["--shell", shell])
                .output()
                .unwrap()
        };

        let before = doctor(&[]);
        assert!(
            !before.status.success(),
            "{shell}: doctor should fail before install"
        );
        assert!(String::from_utf8_lossy(&before.stdout).contains("not installed"));

        let first = doctor(&["--install"]);
        assert_success(&first);
        let first_out = String::from_utf8_lossy(&first.stdout);
        assert!(first_out.contains("Added the"), "{shell}: {first_out}");
        assert!(first_out.contains("source ~/"), "{shell}: {first_out}");
        let installed = fs::read_to_string(&rc).unwrap();
        assert!(installed.starts_with("# existing config\n"));

        let second = doctor(&["--install"]);
        assert_success(&second);
        assert!(String::from_utf8_lossy(&second.stdout).contains("nothing changed"));
        assert_eq!(
            fs::read_to_string(&rc).unwrap(),
            installed,
            "{shell}: rc changed on re-install"
        );

        // A fresh interactive shell picks the functions up from the rc file.
        let probe = if shell == "zsh" {
            "whence -w git-clown git"
        } else {
            "type -t git-clown git"
        };
        let output = sb
            .command(shell)
            .args(["-i", "-c", probe])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.matches("function").count(), 2, "{shell}: {stdout}");
    }
}

#[test]
fn doctor_reports_active_session_through_wrapper() {
    let sb = Sandbox::new();
    let output = sb.run_in_shell("bash", "git clown doctor --shell bash");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("integration loaded (bash)"), "{stdout}");
}
