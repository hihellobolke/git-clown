//! `git-clown doctor`: checks that git-clown and its shell integration are set
//! up, explains what is missing, and can install the integration.

use crate::shell::{self, InstallOutcome, RcState, Shell};
use clap::Args;
use colored::Colorize;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};
use std::{
    env, fs,
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::Command,
};

/// Options for `git-clown doctor`.
#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Install (or update) the shell integration into your shell's rc file
    #[arg(long)]
    pub install: bool,

    /// Shell(s) to check or install for; defaults to the shell in $SHELL
    #[arg(long, value_enum)]
    pub shell: Vec<Shell>,
}

/// Outcome of a single check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Ok,
    Warn,
    Fail,
}

/// A single line of the doctor report.
#[derive(Debug)]
struct Check {
    status: Status,
    label: String,
    detail: String,
    hint: Option<String>,
}

impl Check {
    fn new(status: Status, label: &str, detail: impl Into<String>) -> Self {
        Check {
            status,
            label: label.to_string(),
            detail: detail.into(),
            hint: None,
        }
    }

    fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// A titled group of checks.
#[derive(Debug)]
struct Section {
    title: String,
    checks: Vec<Check>,
}

/// Runs `git-clown doctor` and returns the process exit code: 0 when nothing
/// failed, 1 otherwise.
pub fn run(args: &DoctorArgs) -> i32 {
    if !io::stdout().is_terminal() {
        colored::control::set_override(false);
    }

    let detected = Shell::detect();
    let shells = match (args.shell.is_empty(), detected) {
        (false, _) => args.shell.clone(),
        (true, Some(shell)) => vec![shell],
        (true, None) => vec![],
    };

    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{}\n", "git-clown doctor".bold());

    if args.install {
        for &shell in &shells {
            let _ = writeln!(out, "{}", install_and_describe(shell).1);
        }
        if !shells.is_empty() {
            let _ = writeln!(out);
        }
    }

    let mut sections = vec![environment_section()];
    sections.push(shell_overview_section(detected, &shells));
    sections.extend(shells.iter().map(|&shell| shell_section(shell)));
    let _ = write!(out, "{}", render(&sections));

    let failures = sections
        .iter()
        .flat_map(|s| &s.checks)
        .filter(|c| c.status == Status::Fail)
        .count();
    let fixable: Vec<(Shell, RcState)> = shells
        .iter()
        .filter_map(|&shell| Some((shell, rc_state_of(shell)?)))
        .filter(|(_, state)| matches!(state, RcState::Missing | RcState::Outdated))
        .collect();

    if fixable.is_empty() {
        let _ = writeln!(out, "{}", summary_line(failures));
        return i32::from(failures > 0);
    }

    // Offer to fix the shell integration right away when a human is watching.
    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
    let mut remaining_failures = failures;
    for (shell, state) in fixable {
        let Some(rc) = shell.rc_file() else { continue };
        let verb = if state == RcState::Missing {
            "Install"
        } else {
            "Update"
        };
        let question = format!(
            "{verb} the {} integration in {}?",
            shell.name(),
            shell::tildify(&rc)
        );
        if interactive && confirm(&mut out, &question).unwrap_or(false) {
            let (installed, message) = install_and_describe(shell);
            let _ = writeln!(out, "{message}");
            if installed && state == RcState::Missing {
                remaining_failures = remaining_failures.saturating_sub(1);
            }
        } else {
            let _ = writeln!(
                out,
                "To install it, run: {}",
                format!("git-clown doctor --install --shell {}", shell.name()).cyan()
            );
        }
    }
    let _ = writeln!(out, "\n{}", summary_line(remaining_failures));
    i32::from(remaining_failures > 0)
}

/// Installs `shell`'s integration and describes what changed and how to
/// reload. The flag tells whether the integration is now in place.
fn install_and_describe(shell: Shell) -> (bool, String) {
    let Some(rc) = shell.rc_file() else {
        return (
            false,
            format!("{} Could not determine your home directory", "✗".red()),
        );
    };
    let rc_display = shell::tildify(&rc);
    let reload = format!(
        "Reload it with {} or open a new terminal.",
        format!("source {rc_display}").cyan()
    );
    let message = match shell::install(shell, &rc) {
        Ok(InstallOutcome::Added) => format!(
            "{} Added the {} integration to {}. {reload}",
            "✓".green(),
            shell.name(),
            rc_display.bold()
        ),
        Ok(InstallOutcome::Updated) => format!(
            "{} Updated the {} integration in {}. {reload}",
            "✓".green(),
            shell.name(),
            rc_display.bold()
        ),
        Ok(InstallOutcome::Unchanged) => format!(
            "{} The {} integration is already in {}; nothing changed.",
            "✓".green(),
            shell.name(),
            rc_display.bold()
        ),
        Err(e) => {
            return (
                false,
                format!("{} Could not update {}: {e}", "✗".red(), rc_display.bold()),
            )
        }
    };
    (true, message)
}

/// Asks a yes/no question and reads a single key press (default: no).
fn confirm(out: &mut impl Write, question: &str) -> io::Result<bool> {
    write!(out, "{} {} ", question.bold(), "[y/N]".dimmed())?;
    out.flush()?;

    terminal::enable_raw_mode()?;
    let answer = read_yes_no();
    terminal::disable_raw_mode()?;

    let answer = answer?;
    writeln!(out, "{}", if answer { "yes" } else { "no" })?;
    Ok(answer)
}

/// Waits for a key press in raw mode; `y` means yes, anything else no.
fn read_yes_no() -> io::Result<bool> {
    loop {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            return Ok(match code {
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => false,
                KeyCode::Char('y' | 'Y') => true,
                _ => false,
            });
        }
    }
}

/// Checks for git, the git-clown binary on PATH and the workspace root.
fn environment_section() -> Section {
    Section {
        title: "Environment".to_string(),
        checks: vec![check_git(), check_binary_on_path(), check_workspace_root()],
    }
}

fn check_git() -> Check {
    match Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => Check::new(
            Status::Ok,
            "git",
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ),
        _ => Check::new(Status::Fail, "git", "not found on PATH")
            .hint("Install git: https://git-scm.com/downloads"),
    }
}

fn check_binary_on_path() -> Check {
    let label = "git-clown on PATH";
    let Some(found) = find_on_path("git-clown") else {
        return Check::new(Status::Fail, label, "not found")
            .hint("Put git-clown in a directory on PATH (e.g. `make release` installs to ~/bin), so `git clown` works");
    };
    let running = env::current_exe()
        .ok()
        .and_then(|p| fs::canonicalize(p).ok());
    let resolved = fs::canonicalize(&found).ok();
    let detail = shell::tildify(&found);
    match (running, resolved) {
        (Some(running), Some(resolved)) if running != resolved => {
            Check::new(Status::Warn, label, detail).hint(format!(
                "This doctor is running {}, a different binary",
                shell::tildify(&running)
            ))
        }
        _ => Check::new(Status::Ok, label, detail),
    }
}

fn check_workspace_root() -> Check {
    let label = "workspace root";
    let root = crate::workspace_root_path();
    let source = if env::var_os("WORKSPACE_ROOT").is_some() {
        "from $WORKSPACE_ROOT"
    } else {
        "default; set $WORKSPACE_ROOT to change"
    };
    let detail = format!("{} ({source})", shell::tildify(&root));
    if root.is_dir() {
        Check::new(Status::Ok, label, detail)
    } else if root.exists() {
        Check::new(Status::Fail, label, detail).hint("Exists but is not a directory")
    } else {
        Check::new(Status::Warn, label, detail)
            .hint("Does not exist yet; it is created on the first clone")
    }
}

/// Reports which shell was detected and whether this session has the
/// integration loaded.
fn shell_overview_section(detected: Option<Shell>, shells: &[Shell]) -> Section {
    let shell_var = env::var("SHELL").unwrap_or_default();
    let detected_check = match detected {
        Some(shell) => Check::new(
            Status::Ok,
            "your shell",
            format!("{} ({shell_var})", shell.name()),
        ),
        None if shells.is_empty() => Check::new(
            Status::Warn,
            "your shell",
            if shell_var.is_empty() {
                "$SHELL is not set".to_string()
            } else {
                format!("{shell_var} is not supported")
            },
        )
        .hint("Shell integration supports bash and zsh; pick one with --shell"),
        None => Check::new(
            Status::Warn,
            "your shell",
            format!("{shell_var} is not supported"),
        ),
    };

    let session_check = match env::var(shell::SHELL_ENV) {
        Ok(name) if env::var_os(shell::CD_FILE_ENV).is_some() => Check::new(
            Status::Ok,
            "this session",
            format!("integration loaded ({name}); `git clown <url>` will cd into the clone"),
        ),
        _ => {
            let check = Check::new(
                Status::Warn,
                "this session",
                "integration not loaded in the shell that ran doctor",
            );
            match shells.first().and_then(|s| s.rc_file()) {
                Some(rc) => check.hint(format!(
                    "After installing, run `source {}` or open a new terminal",
                    shell::tildify(&rc)
                )),
                None => check,
            }
        }
    };

    Section {
        title: "Shell".to_string(),
        checks: vec![detected_check, session_check],
    }
}

/// Reads `shell`'s rc file and reports how the integration appears in it.
fn rc_state_of(shell: Shell) -> Option<RcState> {
    let rc = shell.rc_file()?;
    let content = fs::read_to_string(rc).unwrap_or_default();
    Some(shell::rc_state(shell, &content))
}

/// Checks the rc file and runs an end-to-end self-test for `shell`.
fn shell_section(shell: Shell) -> Section {
    let rc_check = match (shell.rc_file(), rc_state_of(shell)) {
        (Some(rc), Some(state)) => {
            let rc = shell::tildify(&rc);
            match state {
                RcState::Current => Check::new(Status::Ok, "rc file", format!("installed in {rc}")),
                RcState::Manual => {
                    Check::new(Status::Ok, "rc file", format!("loaded manually in {rc}"))
                }
                RcState::Outdated => {
                    Check::new(Status::Warn, "rc file", format!("outdated block in {rc}"))
                        .hint("Run `git-clown doctor --install` to refresh it")
                }
                RcState::Missing => {
                    Check::new(Status::Fail, "rc file", format!("not installed in {rc}")).hint(
                        format!(
                            "Run `git-clown doctor --install --shell {}`, or add `{}` to {rc}",
                            shell.name(),
                            shell.load_command()
                        ),
                    )
                }
            }
        }
        _ => Check::new(
            Status::Fail,
            "rc file",
            "could not determine your home directory",
        ),
    };

    let self_test = match find_on_path(shell.name()) {
        None => Check::new(
            Status::Warn,
            "self-test",
            format!("{} not found on PATH, skipped", shell.name()),
        ),
        Some(_) => match env::current_exe() {
            Ok(exe) => match self_test(shell, &exe) {
                Ok(()) => Check::new(
                    Status::Ok,
                    "self-test",
                    "`git-clown` and `git clown` wrappers cd correctly",
                ),
                Err(e) => Check::new(Status::Fail, "self-test", e),
            },
            Err(e) => Check::new(Status::Warn, "self-test", format!("skipped: {e}")),
        },
    };

    Section {
        title: format!("Shell integration ({})", shell.name()),
        checks: vec![rc_check, self_test],
    }
}

/// Loads the integration into a fresh, rc-less `shell` and checks that both
/// `git-clown` and `git clown` cd into the directory reported by the binary
/// at `exe` (via the hidden `__selftest-cd` subcommand).
pub fn self_test(shell: Shell, exe: &Path) -> Result<(), String> {
    let target = tempfile::tempdir().map_err(|e| e.to_string())?;
    let target = fs::canonicalize(target.path()).map_err(|e| e.to_string())?;

    let script = format!(
        "{}\n\
         cd / || exit 90\n\
         git-clown __selftest-cd \"$1\" || exit 91\n\
         [ \"$PWD\" = \"$1\" ] || {{ echo \"git-clown left the shell in $PWD\" >&2; exit 92; }}\n\
         cd / || exit 90\n\
         git clown __selftest-cd \"$1\" || exit 93\n\
         [ \"$PWD\" = \"$1\" ] || {{ echo \"git clown left the shell in $PWD\" >&2; exit 94; }}\n",
        shell.init_script()
    );
    let no_rc: &[&str] = match shell {
        Shell::Bash => &["--norc", "--noprofile"],
        Shell::Zsh => &["-f"],
    };

    let output = Command::new(shell.name())
        .args(no_rc)
        .arg("-c")
        .arg(&script)
        .arg("git-clown-selftest")
        .arg(&target)
        .env("PATH", path_with_exe_dir(exe))
        .env_remove("BASH_ENV")
        .env_remove(shell::CD_FILE_ENV)
        .env_remove(shell::SHELL_ENV)
        .output()
        .map_err(|e| format!("could not run {}: {e}", shell.name()))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "failed ({}): {}",
            output.status,
            stderr.lines().last().unwrap_or("no output").trim()
        ))
    }
}

/// `$PATH` with the directory of `exe` prepended, so the wrapper's
/// `command git-clown` runs the binary under test.
fn path_with_exe_dir(exe: &Path) -> std::ffi::OsString {
    let current = env::var_os("PATH").unwrap_or_default();
    let dirs = exe
        .parent()
        .map(Path::to_path_buf)
        .into_iter()
        .chain(env::split_paths(&current));
    env::join_paths(dirs).unwrap_or(current)
}

/// Finds an executable named `name` on `$PATH`.
fn find_on_path(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// Renders the report sections as aligned, colored text.
fn render(sections: &[Section]) -> String {
    let width = sections
        .iter()
        .flat_map(|s| &s.checks)
        .map(|c| c.label.len())
        .max()
        .unwrap_or(0);

    sections
        .iter()
        .map(|section| {
            let lines: String = section
                .checks
                .iter()
                .map(|check| {
                    let icon = match check.status {
                        Status::Ok => "✓".green(),
                        Status::Warn => "!".yellow(),
                        Status::Fail => "✗".red(),
                    };
                    let hint = check
                        .hint
                        .as_ref()
                        .map(|h| format!("      {} {}\n", "↳".dimmed(), h.dimmed()))
                        .unwrap_or_default();
                    format!("  {icon} {:<width$}  {}\n{hint}", check.label, check.detail)
                })
                .collect();
            format!("{}\n{lines}\n", section.title.bold().underline())
        })
        .collect()
}

/// The closing line of the report.
fn summary_line(failures: usize) -> String {
    match failures {
        0 => "Everything looks good.".green().bold().to_string(),
        1 => "1 problem found.".red().bold().to_string(),
        n => format!("{n} problems found.").red().bold().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_aligns_labels_and_shows_hints() {
        colored::control::set_override(false);
        let sections = vec![Section {
            title: "Env".to_string(),
            checks: vec![
                Check::new(Status::Ok, "git", "git version 2.0"),
                Check::new(Status::Fail, "long label", "missing").hint("fix it"),
            ],
        }];
        let text = render(&sections);
        assert!(text.contains("  ✓ git         git version 2.0\n"));
        assert!(text.contains("  ✗ long label  missing\n"));
        assert!(text.contains("↳ fix it"));
    }

    #[test]
    fn test_summary_line() {
        colored::control::set_override(false);
        assert_eq!(summary_line(0), "Everything looks good.");
        assert_eq!(summary_line(1), "1 problem found.");
        assert_eq!(summary_line(3), "3 problems found.");
    }

    #[test]
    fn test_find_on_path_finds_sh() {
        assert!(find_on_path("sh").is_some());
        assert!(find_on_path("definitely-not-a-real-binary-xyz").is_none());
    }
}
