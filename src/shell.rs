//! Shell integration: lets `git clown <url>` leave the user's shell inside the
//! cloned repository.
//!
//! A child process cannot change its parent shell's working directory, so the
//! shell has to do the `cd` itself. `git-clown init <shell>` prints shell
//! functions that wrap `git-clown` (and `git clown`). The wrapper runs the
//! binary with [`CD_FILE_ENV`] pointing at a temp file; after a successful
//! clone the binary writes the target directory into that file (see
//! [`write_cd_target`]) and the wrapper `cd`s into it. stdout and stderr are
//! never used for this, so normal output is unaffected.

use clap::ValueEnum;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

/// Environment variable naming the file the binary writes its cd target to.
pub const CD_FILE_ENV: &str = "GIT_CLOWN_CD_FILE";

/// Environment variable the wrapper sets to the name of the shell it runs in.
pub const SHELL_ENV: &str = "GIT_CLOWN_SHELL";

/// First line of the block `doctor --install` manages in rc files.
const BLOCK_START: &str = "# >>> git-clown shell integration >>>";

/// Last line of the block `doctor --install` manages in rc files.
const BLOCK_END: &str = "# <<< git-clown shell integration <<<";

/// Wrapper functions shared by bash and zsh; `__GIT_CLOWN_SHELL__` is replaced
/// with the shell name.
const POSIX_SCRIPT_TEMPLATE: &str = include_str!("../shell/git-clown.sh");

/// Shells git-clown can integrate with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
}

impl Shell {
    /// The shell's executable / display name.
    pub fn name(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
        }
    }

    /// Maps a shell path such as `/bin/zsh` or a login name such as `-bash`
    /// to a supported shell.
    pub fn from_path(path: &str) -> Option<Shell> {
        let name = Path::new(path)
            .file_name()?
            .to_str()?
            .trim_start_matches('-');
        match name {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            _ => None,
        }
    }

    /// Detects the user's shell from `$SHELL`.
    pub fn detect() -> Option<Shell> {
        env::var("SHELL").ok().as_deref().and_then(Shell::from_path)
    }

    /// The rc file integration is installed into, based on the current
    /// environment (`$HOME`, `$ZDOTDIR`).
    pub fn rc_file(self) -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let zdotdir = env::var_os("ZDOTDIR").map(PathBuf::from);
        Some(self.rc_file_in(&home, zdotdir.as_deref()))
    }

    /// The rc file for this shell given a home directory and optional `$ZDOTDIR`.
    ///
    /// bash on macOS starts login shells, which read `~/.bash_profile` rather
    /// than `~/.bashrc`.
    pub fn rc_file_in(self, home: &Path, zdotdir: Option<&Path>) -> PathBuf {
        match self {
            Shell::Zsh => zdotdir.unwrap_or(home).join(".zshrc"),
            Shell::Bash if cfg!(target_os = "macos") => home.join(".bash_profile"),
            Shell::Bash => home.join(".bashrc"),
        }
    }

    /// The shell functions printed by `git-clown init <shell>`.
    pub fn init_script(self) -> String {
        POSIX_SCRIPT_TEMPLATE.replace("__GIT_CLOWN_SHELL__", self.name())
    }

    /// The command that loads the integration, as written into rc files.
    pub fn load_command(self) -> String {
        format!("eval \"$(command git-clown init {})\"", self.name())
    }

    /// The marked block `doctor --install` writes into the rc file.
    pub fn rc_block(self) -> String {
        format!(
            "{BLOCK_START}\n\
             # Lets `git clown <url>` cd into the cloned repo. Managed by `git-clown doctor --install`.\n\
             if command -v git-clown >/dev/null 2>&1; then\n  \
             {}\n\
             fi\n\
             {BLOCK_END}\n",
            self.load_command()
        )
    }
}

/// How the integration appears in an rc file.
#[derive(Debug, PartialEq, Eq)]
pub enum RcState {
    /// No git-clown integration found.
    Missing,
    /// The managed block is present and up to date.
    Current,
    /// The managed block is present but differs from what this version writes.
    Outdated,
    /// No managed block, but the user loads `git-clown init` themselves.
    Manual,
}

/// Result of installing the integration into an rc file.
#[derive(Debug, PartialEq, Eq)]
pub enum InstallOutcome {
    /// The block was appended to the rc file (created if needed).
    Added,
    /// An outdated managed block was replaced.
    Updated,
    /// Nothing changed: the integration was already present.
    Unchanged,
}

/// Locates the managed block in `content`, returning its byte range
/// (including the end marker line and its newline, if any).
fn find_block(content: &str) -> Option<std::ops::Range<usize>> {
    let start = content.find(BLOCK_START)?;
    let end_marker = start + content[start..].find(BLOCK_END)?;
    let mut end = end_marker + BLOCK_END.len();
    if content[end..].starts_with('\n') {
        end += 1;
    }
    Some(start..end)
}

/// Determines how `shell`'s integration appears in rc file `content`.
pub fn rc_state(shell: Shell, content: &str) -> RcState {
    match find_block(content) {
        Some(range) if content[range.clone()].trim_end() == shell.rc_block().trim_end() => {
            RcState::Current
        }
        Some(_) => RcState::Outdated,
        None if loads_manually(shell, content) => RcState::Manual,
        None => RcState::Missing,
    }
}

/// Whether a non-comment part of `content` runs `git-clown init <shell>`.
fn loads_manually(shell: Shell, content: &str) -> bool {
    let command = format!("git-clown init {}", shell.name());
    content
        .lines()
        .any(|line| strip_comment(line).contains(&command))
}

/// Drops a shell comment: a `#` at the start of a word and everything after it.
fn strip_comment(line: &str) -> &str {
    let comment_start = line
        .char_indices()
        .find(|&(i, c)| {
            c == '#'
                && line[..i]
                    .chars()
                    .next_back()
                    .is_none_or(char::is_whitespace)
        })
        .map_or(line.len(), |(i, _)| i);
    &line[..comment_start]
}

/// Returns `content` with `shell`'s managed block added or refreshed, along
/// with what changed. Pure, so it is easy to test for idempotency.
pub fn upsert_block(shell: Shell, content: &str) -> (String, InstallOutcome) {
    let block = shell.rc_block();
    match (rc_state(shell, content), find_block(content)) {
        (RcState::Current | RcState::Manual, _) => (content.to_string(), InstallOutcome::Unchanged),
        (RcState::Outdated, Some(range)) => {
            let mut updated = content.to_string();
            updated.replace_range(range, &block);
            (updated, InstallOutcome::Updated)
        }
        _ => {
            let separator = match content {
                "" => "",
                c if c.ends_with("\n\n") => "",
                c if c.ends_with('\n') => "\n",
                _ => "\n\n",
            };
            (
                format!("{content}{separator}{block}"),
                InstallOutcome::Added,
            )
        }
    }
}

/// Installs `shell`'s integration into `rc_path`, creating the file if needed.
/// Running it again is a no-op.
pub fn install(shell: Shell, rc_path: &Path) -> io::Result<InstallOutcome> {
    let content = match fs::read_to_string(rc_path) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };
    let (updated, outcome) = upsert_block(shell, &content);
    if outcome != InstallOutcome::Unchanged {
        if let Some(parent) = rc_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(rc_path, updated)?;
    }
    Ok(outcome)
}

/// Writes `dir` (made absolute) to the file named by [`CD_FILE_ENV`], so the
/// shell wrapper can `cd` there. Returns `Ok(false)` when not running under
/// the wrapper.
///
/// The file holds the exact path bytes with no terminator, so paths
/// containing newlines (or ending in one) survive; the wrapper reads the
/// whole file.
pub fn write_cd_target(dir: &Path) -> io::Result<bool> {
    match env::var_os(CD_FILE_ENV) {
        Some(file) if !file.is_empty() => {
            let dir = std::path::absolute(dir)?;
            fs::write(file, path_bytes(&dir))?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[cfg(unix)]
fn path_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn path_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().into_owned().into_bytes()
}

/// Replaces a leading home directory with `~` for display.
pub fn tildify(path: &Path) -> String {
    match dirs::home_dir().and_then(|home| path.strip_prefix(home).ok().map(Path::to_path_buf)) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_from_path() {
        assert_eq!(Shell::from_path("/bin/zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::from_path("/usr/local/bin/bash"), Some(Shell::Bash));
        assert_eq!(Shell::from_path("-zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::from_path("/usr/bin/fish"), None);
        assert_eq!(Shell::from_path(""), None);
    }

    #[test]
    fn test_rc_file_in() {
        let home = Path::new("/home/u");
        assert_eq!(Shell::Zsh.rc_file_in(home, None), home.join(".zshrc"));
        assert_eq!(
            Shell::Zsh.rc_file_in(home, Some(Path::new("/cfg/zsh"))),
            PathBuf::from("/cfg/zsh/.zshrc")
        );
        let bash_rc = if cfg!(target_os = "macos") {
            ".bash_profile"
        } else {
            ".bashrc"
        };
        assert_eq!(Shell::Bash.rc_file_in(home, None), home.join(bash_rc));
    }

    #[test]
    fn test_init_script_names_shell() {
        let script = Shell::Zsh.init_script();
        assert!(script.contains("GIT_CLOWN_SHELL=zsh"));
        assert!(script.contains(CD_FILE_ENV));
        assert!(!script.contains("__GIT_CLOWN_SHELL__"));
    }

    #[test]
    fn test_upsert_block_adds_once() {
        let original = "export PATH=$HOME/bin:$PATH\n";
        let (once, outcome) = upsert_block(Shell::Zsh, original);
        assert_eq!(outcome, InstallOutcome::Added);
        assert!(once.starts_with(original));
        assert!(once.contains(&Shell::Zsh.load_command()));
        assert_eq!(rc_state(Shell::Zsh, &once), RcState::Current);

        let (twice, outcome) = upsert_block(Shell::Zsh, &once);
        assert_eq!(outcome, InstallOutcome::Unchanged);
        assert_eq!(twice, once);
    }

    #[test]
    fn test_upsert_block_separators() {
        assert!(upsert_block(Shell::Bash, "").0.starts_with(BLOCK_START));
        assert!(upsert_block(Shell::Bash, "x").0.starts_with("x\n\n# >>>"));
        assert!(upsert_block(Shell::Bash, "x\n").0.starts_with("x\n\n# >>>"));
        assert!(upsert_block(Shell::Bash, "x\n\n")
            .0
            .starts_with("x\n\n# >>>"));
    }

    #[test]
    fn test_upsert_block_replaces_outdated_block() {
        let content = format!("before\n{BLOCK_START}\nold stuff\n{BLOCK_END}\nafter\n");
        assert_eq!(rc_state(Shell::Zsh, &content), RcState::Outdated);

        let (updated, outcome) = upsert_block(Shell::Zsh, &content);
        assert_eq!(outcome, InstallOutcome::Updated);
        assert!(!updated.contains("old stuff"));
        assert!(updated.starts_with("before\n"));
        assert!(updated.ends_with("after\n"));
        assert_eq!(rc_state(Shell::Zsh, &updated), RcState::Current);
    }

    #[test]
    fn test_manual_setup_is_respected() {
        let content = "eval \"$(git-clown init zsh)\"\n";
        assert_eq!(rc_state(Shell::Zsh, content), RcState::Manual);
        assert_eq!(rc_state(Shell::Bash, content), RcState::Missing);
        assert_eq!(
            upsert_block(Shell::Zsh, content).1,
            InstallOutcome::Unchanged
        );
    }

    #[test]
    fn test_commented_setup_is_not_installed() {
        for shell in [Shell::Bash, Shell::Zsh] {
            let name = shell.name();
            for content in [
                format!("# To install later: eval \"$(git-clown init {name})\"\n"),
                format!("   #eval \"$(git-clown init {name})\"\n"),
                format!("echo hi # git-clown init {name}\n"),
            ] {
                assert_eq!(rc_state(shell, &content), RcState::Missing, "{content}");
                assert_eq!(upsert_block(shell, &content).1, InstallOutcome::Added);
            }
            let trailing = format!("eval \"$(git-clown init {name})\" # load git-clown\n");
            assert_eq!(rc_state(shell, &trailing), RcState::Manual);
            let in_word = format!("eval \"$(git-clown init {name})\"#x\n");
            assert_eq!(rc_state(shell, &in_word), RcState::Manual);
        }
    }

    #[test]
    fn test_install_creates_and_is_idempotent() {
        let dir = tempdir().unwrap();
        let rc = dir.path().join("nested").join(".zshrc");

        assert_eq!(install(Shell::Zsh, &rc).unwrap(), InstallOutcome::Added);
        let first = fs::read_to_string(&rc).unwrap();
        assert_eq!(install(Shell::Zsh, &rc).unwrap(), InstallOutcome::Unchanged);
        assert_eq!(fs::read_to_string(&rc).unwrap(), first);
    }
}
