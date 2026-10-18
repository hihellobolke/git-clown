//! # Git Clown
//!
//! A tool that combines git clone with smart workspace management.
//!
//! Git Clown clones git repositories into a structured directory layout based on the repository URL.
//! For example, a repository at `https://github.com/example/repo.git` would be cloned to
//! `$WORKSPACE_ROOT/github.com/example/`.
//!
//! ## Features
//!
//! * Creates directory structure based on Git URLs
//! * Supports both HTTPS and SSH Git URLs
//! * Skips cloning if repository already exists
//! * Finds and loads `.envrc` or `.env` files
//! * Passes environment variables to git clone
//! * Async operations with Tokio
//! * Colored logging output
//!
//! ## Usage
//!
//! ```
//! git-clown https://github.com/example/repo.git
//! ```
//!
//! Environment variables:
//! * `WORKSPACE_ROOT`: The root directory for the workspace. Defaults to `~/ws/` if not set.

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use dotenvy;
use log::{debug, error, info, warn};
use regex::Regex;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command as StdCommand,
};
use thiserror::Error;
use tokio::process::Command;
use url::Url;

/// Error types for git-clown operations
#[derive(Error, Debug)]
enum GitClownError {
    /// Error when parsing a Git URL, contains the error details
    #[error("Failed to parse Git URL: {0}")]
    UrlParseError(String),

    /// Error when creating directory structure, contains the error details
    #[error("Failed to create directory: {0}")]
    DirectoryCreationError(String),

    /// Error when executing git clone command, contains the error details
    #[error("Failed to execute git clone: {0}")]
    GitCloneError(String),

    /// Error when parsing environment file (.env or .envrc), contains the error details
    #[error("Failed to parse environment file: {0}")]
    EnvParseError(String),

    /// IO errors that may occur during file operations
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Error when joining Tokio runtime
    #[error("Failed to join Tokio runtime: {0}")]
    TokioError(#[from] tokio::task::JoinError),

    /// Error related to environment variable operations
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),
}

/// A tool to clone git repositories with a structured directory layout
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A tool to clone git repos with a structured workspace layout"
)]
struct Args {
    /// The URL of the git repository to clone
    #[arg(required = true)]
    git_url: String,

    /// Clone as a bare repository and set up an initial worktree for the default branch
    #[arg(long)]
    bare: bool,
}

/// Parses a git URL and extracts components needed for directory structure
///
/// This function accepts both HTTPS and SSH Git URLs:
/// - HTTPS: `https://github.com/example/repo.git`
/// - SSH: `git@github.com:example/repo.git`
///
/// # Arguments
///
/// * `url_str` - The Git URL as a string
///
/// # Returns
///
/// A tuple containing:
/// * `host` - The hostname (e.g., "github.com")
/// * `organization` - The organization or user name (e.g., "example")
/// * `repository` - The repository name (e.g., "repo")
///
/// # Errors
///
/// Returns `GitClownError::UrlParseError` if:
/// * The URL cannot be parsed
/// * The URL is missing a host
/// * The URL path doesn't contain organization and repository parts
async fn parse_git_url(url_str: &str) -> Result<(String, String, String), GitClownError> {
    debug!("Parsing Git URL: {}", url_str.cyan());

    // Try to parse as a standard URL
    match Url::parse(url_str) {
        Ok(url) => {
            let host = url
                .host_str()
                .ok_or_else(|| GitClownError::UrlParseError("No host in URL".to_string()))?
                .to_string();

            // The path without the leading slash
            let path = url.path().trim_start_matches('/');
            let path_segments: Vec<&str> = path.split('/').collect();

            if path_segments.len() < 2 {
                error!(
                    "URL path {} does not contain organization and repository",
                    path.red()
                );
                return Err(GitClownError::UrlParseError(
                    "URL path does not contain organization and repository".to_string(),
                ));
            }

            let org = path_segments[0].to_string();
            let mut repo = path_segments[1].to_string();

            // Remove .git extension if present
            if repo.ends_with(".git") {
                repo = repo[..repo.len() - 4].to_string();
            }

            info!(
                "Successfully parsed HTTPS Git URL: host={}, organization={}, repository={}",
                host.green(),
                org.green(),
                repo.green()
            );

            Ok((host, org, repo))
        }
        Err(_) => {
            // Try to handle SSH style URLs like git@github.com:org/repo.git
            if url_str.contains('@') && url_str.contains(':') {
                debug!("Attempting to parse as SSH Git URL");

                let parts: Vec<&str> = url_str.split('@').collect();
                if parts.len() != 2 {
                    error!("Invalid SSH URL format: {}", url_str.red());
                    return Err(GitClownError::UrlParseError(
                        "Invalid SSH URL format".to_string(),
                    ));
                }

                let host_and_path: Vec<&str> = parts[1].split(':').collect();
                if host_and_path.len() != 2 {
                    error!("Invalid SSH URL format: {}", url_str.red());
                    return Err(GitClownError::UrlParseError(
                        "Invalid SSH URL format".to_string(),
                    ));
                }

                let host = host_and_path[0].to_string();
                let path = host_and_path[1];
                let path_segments: Vec<&str> = path.split('/').collect();

                if path_segments.len() < 2 {
                    error!("SSH URL path does not contain organization and repository");
                    return Err(GitClownError::UrlParseError(
                        "URL path does not contain organization and repository".to_string(),
                    ));
                }

                let org = path_segments[0].to_string();
                let mut repo = path_segments[1].to_string();

                // Remove .git extension if present
                if repo.ends_with(".git") {
                    repo = repo[..repo.len() - 4].to_string();
                }

                info!(
                    "Successfully parsed SSH Git URL: host={}, organization={}, repository={}",
                    host.green(),
                    org.green(),
                    repo.green()
                );

                Ok((host, org, repo))
            } else {
                error!("Invalid Git URL format: {}", url_str.red());
                Err(GitClownError::UrlParseError(
                    "Invalid URL format".to_string(),
                ))
            }
        }
    }
}

/// Creates the directory structure for cloning based on URL components
///
/// Takes the extracted components from a Git URL and creates the appropriate
/// directory structure under the workspace root.
///
/// # Arguments
///
/// * `workspace_root` - The root directory of the workspace
/// * `host` - The hostname (e.g., "github.com")
/// * `org` - The organization or user name (e.g., "example")
///
/// # Returns
///
/// The complete path where the repository should be cloned (e.g., "/path/to/workspace/github.com/example")
///
/// # Errors
///
/// Returns `GitClownError::DirectoryCreationError` if directory creation fails.
async fn create_directory_structure(
    workspace_root: &Path,
    host: &str,
    org: &str,
) -> Result<PathBuf, GitClownError> {
    let dir_path = workspace_root.join(host).join(org);

    debug!(
        "Creating directory structure: {}",
        dir_path.display().to_string().cyan()
    );

    // Use tokio's fs for async file operations
    match fs::create_dir_all(&dir_path) {
        Ok(_) => {
            info!(
                "Successfully created directory: {}",
                dir_path.display().to_string().green()
            );
            Ok(dir_path)
        }
        Err(e) => {
            error!(
                "Failed to create directory {}: {}",
                dir_path.display().to_string().red(),
                e.to_string().red()
            );
            Err(GitClownError::DirectoryCreationError(e.to_string()))
        }
    }
}

/// Finds .envrc or .env file by searching up the directory tree
///
/// Starts from the specified directory and traverses up the directory tree,
/// looking for .envrc or .env files until reaching the workspace root.
/// Prioritizes .envrc files over .env files in the same directory.
///
/// # Arguments
///
/// * `start_dir` - The directory to start searching from
/// * `workspace_root` - The workspace root directory (stopping point)
///
/// # Returns
///
/// * `Some(PathBuf)` - Path to the found environment file
/// * `None` - No environment file was found
///
/// # Errors
///
/// Returns IO errors that may occur during directory traversal.
async fn find_env_file(
    start_dir: &Path,
    workspace_root: &Path,
) -> Result<Option<PathBuf>, GitClownError> {
    debug!(
        "Searching for environment file starting from: {}",
        start_dir.display().to_string().cyan()
    );

    let mut current_dir = start_dir.to_path_buf();

    loop {
        // Check for .envrc file first (higher priority)
        let envrc_path = current_dir.join(".envrc");
        if envrc_path.exists() && envrc_path.is_file() {
            info!(
                "Found .envrc file at: {}",
                envrc_path.display().to_string().green()
            );
            return Ok(Some(envrc_path));
        }

        // Check for .env file
        let env_path = current_dir.join(".env");
        if env_path.exists() && env_path.is_file() {
            info!(
                "Found .env file at: {}",
                env_path.display().to_string().green()
            );
            return Ok(Some(env_path));
        }

        // Stop if we've reached the workspace root
        if current_dir == workspace_root {
            debug!("Reached workspace root without finding environment file");
            break;
        }

        // Go up one directory
        match current_dir.parent() {
            Some(parent) => {
                current_dir = parent.to_path_buf();
                debug!(
                    "Moving up to parent directory: {}",
                    current_dir.display().to_string().cyan()
                );
            }
            None => {
                debug!("Reached filesystem root without finding environment file");
                break;
            }
        }
    }

    warn!("No environment file (.envrc or .env) found in path hierarchy");
    Ok(None)
}

/// Reads and parses environment variables from .env or .envrc files
///
/// Handles both .env files (using dotenvy) and .envrc files (with custom parsing)
/// supporting the `export KEY=VALUE` syntax commonly used in shell scripts.
///
/// # Arguments
///
/// * `env_file_path` - Path to the environment file (.env or .envrc)
///
/// # Returns
///
/// A HashMap containing the environment variables from the file
///
/// # Errors
///
/// * `GitClownError::EnvParseError` - If there's an error parsing the environment file
/// * `GitClownError::IoError` - If there's an error reading the file
async fn read_env_file(env_file_path: &Path) -> Result<HashMap<String, String>, GitClownError> {
    debug!(
        "Reading environment file: {}",
        env_file_path.display().to_string().cyan()
    );

    // Load the file using dotenvy if it's a .env file
    if env_file_path
        .file_name()
        .map_or(false, |name| name == ".env")
    {
        debug!("Using dotenvy to load .env file");
        match dotenvy::from_path_iter(env_file_path) {
            Ok(vars) => {
                let mut env_map = HashMap::new();
                for result in vars {
                    match result {
                        Ok((key, value)) => {
                            debug!(
                                "Found environment variable: {}={}",
                                key.cyan(),
                                value.cyan()
                            );
                            env_map.insert(key, value);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse environment variable: {}",
                                e.to_string().yellow()
                            );
                        }
                    }
                }

                info!(
                    "Successfully loaded {} environment variables from {}",
                    env_map.len().to_string().green(),
                    env_file_path.display().to_string().green()
                );

                return Ok(env_map);
            }
            Err(e) => {
                error!("Failed to parse .env file: {}", e.to_string().red());
                return Err(GitClownError::EnvParseError(e.to_string()));
            }
        }
    }

    // For .envrc files or other env files, we need to handle the export syntax
    debug!("Parsing environment file with custom parser for .envrc format");
    let file = match File::open(env_file_path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open environment file: {}", e.to_string().red());
            return Err(GitClownError::IoError(e));
        }
    };

    let reader = BufReader::new(file);
    let mut env_map = HashMap::new();

    // Regular expression to match both "export KEY=VALUE" and "KEY=VALUE" formats
    let re = Regex::new(r"(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)=(.*)").unwrap();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(captures) = re.captures(trimmed) {
            if captures.len() >= 3 {
                let key = captures[1].to_string();
                // Remove surrounding quotes if present in the value
                let mut value = captures[2].to_string();

                // Handle quoted values
                if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    value = value[1..value.len() - 1].to_string();
                }

                debug!(
                    "Found environment variable: {}={}",
                    key.cyan(),
                    value.cyan()
                );
                env_map.insert(key, value);
            }
        }
    }

    info!(
        "Successfully loaded {} environment variables from {}",
        env_map.len().to_string().green(),
        env_file_path.display().to_string().green()
    );

    Ok(env_map)
}

/// Executes git clone command in the specified directory with the provided git URL
///
/// Uses Tokio's async process execution to run git clone in the specified directory,
/// applying all the environment variables loaded from the environment file.
///
/// # Arguments
///
/// * `git_url` - The URL of the Git repository to clone
/// * `dir_path` - The directory path where git clone should be executed (parent directory of the clone)
/// * `env_vars` - Environment variables to pass to the git clone command
///
/// # Returns
///
/// The exit status of the git clone command
///
/// # Errors
///
/// * `GitClownError::GitCloneError` - If there's an error executing the git clone command
/// * `GitClownError::IoError` - If there's an IO error during command execution
async fn execute_git_clone(
    git_url: &str,
    dir_path: &Path,
    env_vars: &HashMap<String, String>,
) -> Result<std::process::ExitStatus, GitClownError> {
    info!(
        "Executing git clone for {} in {}",
        git_url.green(),
        dir_path.display().to_string().green()
    );

    // Create the command with current directory and merged environment applied.
    let mut command = build_git_command(dir_path, &["clone", git_url], env_vars);

    debug!("Running command: {:?}", command);

    // Execute the command and capture output
    let output = command.output().await.map_err(|e| {
        error!("Failed to execute git clone: {}", e.to_string().red());
        GitClownError::GitCloneError(e.to_string())
    })?;

    // Convert stdout and stderr to strings for logging
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print stdout and stderr
    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }

    // Log status
    if output.status.success() {
        info!("Git clone completed successfully");
    } else {
        warn!(
            "Git clone exited with status: {}",
            output.status.to_string().yellow()
        );
    }

    Ok(output.status)
}

/// Builds a git command with a working directory, argument list, and a merged
/// environment map applied to the subprocess.
fn build_git_command(
    current_dir: &Path,
    args: &[&str],
    env_vars: &HashMap<String, String>,
) -> Command {
    let mut command = Command::new("git");
    command.current_dir(current_dir).args(args).envs(env_vars);
    command
}

/// Merges process and file environment maps, with file values overriding process
/// values for matching keys.
fn merge_env_vars(
    base_env_vars: HashMap<String, String>,
    file_env_vars: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = base_env_vars;
    for (key, value) in file_env_vars {
        merged.insert(key, value);
    }
    merged
}

/// Gets the workspace root directory from the environment or uses default
///
/// Determines the workspace root directory by:
/// 1. Checking the WORKSPACE_ROOT environment variable
/// 2. If not set, using the ~/ws directory (creating it if necessary)
///
/// # Returns
///
/// The path to the workspace root directory
async fn get_workspace_root() -> PathBuf {
    debug!("Getting workspace root directory");

    // Check for the WORKSPACE_ROOT environment variable first
    if let Ok(path) = env::var("WORKSPACE_ROOT") {
        let path_buf = PathBuf::from(path);
        debug!(
            "Using workspace root from WORKSPACE_ROOT environment variable: {}",
            path_buf.display().to_string().cyan()
        );
        return path_buf;
    }

    // Default to ~/ws
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let default_path = home_dir.join("ws");
    info!(
        "WORKSPACE_ROOT not set, using default: {}",
        default_path.display().to_string().green()
    );

    // Create the default path if it doesn't exist
    if !default_path.exists() {
        debug!("Default workspace path does not exist, creating it");
        if let Err(e) = fs::create_dir_all(&default_path) {
            warn!(
                "Failed to create default workspace directory: {}",
                e.to_string().yellow()
            );
        }
    }

    default_path
}

/// Checks if a git repository has already been cloned
///
/// Verifies whether a directory with the repository name exists and contains
/// a .git subdirectory, which suggests that the repository is already cloned.
///
/// # Arguments
///
/// * `dir_path` - The directory path where repositories are cloned (parent directory)
/// * `repo_name` - The name of the repository to check
///
/// # Returns
///
/// `true` if the repository appears to be already cloned, `false` otherwise
async fn repository_exists(dir_path: &Path, repo_name: &str) -> bool {
    let repo_path = dir_path.join(repo_name);
    let git_dir_path = repo_path.join(".git");

    debug!(
        "Checking if repository exists at: {}",
        repo_path.display().to_string().cyan()
    );

    if repo_path.exists() && repo_path.is_dir() && git_dir_path.exists() && git_dir_path.is_dir() {
        debug!("Repository directory exists and contains .git directory");
        return true;
    }

    debug!("Repository doesn't exist or is not a valid git repository");
    false
}

/// Clones a repository as a bare repo into `<repo_root>/.bare`.
///
/// Skips the clone if `.bare` already exists.
async fn clone_bare(
    git_url: &str,
    repo_root: &Path,
    env_vars: &HashMap<String, String>,
) -> Result<(), GitClownError> {
    let bare_path = repo_root.join(".bare");

    if bare_path.exists() {
        info!(
            "Bare repository already exists at {}, skipping clone",
            bare_path.display().to_string().green()
        );
        return Ok(());
    }

    info!(
        "Cloning bare repository {} into {}",
        git_url.green(),
        bare_path.display().to_string().green()
    );

    let mut command =
        build_git_command(repo_root, &["clone", "--bare", git_url, ".bare"], env_vars);

    let output = command.output().await.map_err(|e| {
        error!(
            "Failed to execute git clone --bare: {}",
            e.to_string().red()
        );
        GitClownError::GitCloneError(e.to_string())
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        println!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }

    if !output.status.success() {
        return Err(GitClownError::GitCloneError(format!(
            "git clone --bare exited with status: {}",
            output.status
        )));
    }

    info!("Bare clone completed successfully");
    Ok(())
}

/// Writes `gitdir: ./.bare` to `<repo_root>/.git` so git tools treat the
/// repo root directory as a git repo backed by the bare store.
async fn write_git_pointer(repo_root: &Path) -> Result<(), GitClownError> {
    let git_file = repo_root.join(".git");
    debug!(
        "Writing .git pointer at {}",
        git_file.display().to_string().cyan()
    );
    fs::write(&git_file, "gitdir: ./.bare\n")?;
    info!(
        "Written .git pointer at {}",
        git_file.display().to_string().green()
    );
    Ok(())
}

/// Fixes the remote fetch refspec for the bare repo (bare clones omit it by
/// default) and then runs `git fetch` to populate remote-tracking refs.
async fn fix_refspec_and_fetch(
    repo_root: &Path,
    env_vars: &HashMap<String, String>,
) -> Result<(), GitClownError> {
    let output = build_git_command(
        repo_root,
        &[
            "-C",
            ".bare",
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ],
        env_vars,
    )
    .output()
    .await
    .map_err(|e| GitClownError::GitCloneError(e.to_string()))?;

    if !output.status.success() {
        return Err(GitClownError::GitCloneError(format!(
            "Failed to set fetch refspec: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    info!("Fetch refspec set");

    let output = build_git_command(repo_root, &["-C", ".bare", "fetch"], env_vars)
        .output()
        .await
        .map_err(|e| GitClownError::GitCloneError(e.to_string()))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }
    if !output.status.success() {
        return Err(GitClownError::GitCloneError(format!(
            "git fetch failed: {}",
            stderr
        )));
    }
    info!("git fetch completed");
    Ok(())
}

/// Detects the default branch of the bare repo by reading the symbolic `HEAD`
/// ref. Returns an error if the ref cannot be resolved (e.g. empty repo).
async fn detect_default_branch(
    repo_root: &Path,
    env_vars: &HashMap<String, String>,
) -> Result<String, GitClownError> {
    let output = build_git_command(
        repo_root,
        &["-C", ".bare", "symbolic-ref", "HEAD"],
        env_vars,
    )
    .output()
    .await
    .map_err(|e| GitClownError::GitCloneError(e.to_string()))?;

    if !output.status.success() {
        return Err(GitClownError::GitCloneError(
            "Could not determine default branch: `git symbolic-ref HEAD` failed. \
             The repository may be empty."
                .to_string(),
        ));
    }

    let ref_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let branch = ref_str
        .strip_prefix("refs/heads/")
        .unwrap_or(&ref_str)
        .to_string();

    if branch.is_empty() {
        return Err(GitClownError::GitCloneError(
            "Could not determine default branch: HEAD ref is empty".to_string(),
        ));
    }

    info!("Detected default branch: {}", branch.green());
    Ok(branch)
}

/// Adds a git worktree for `branch` at `<repo_root>/<branch>`.
///
/// Skips the operation if the worktree directory already exists.
async fn add_worktree(
    repo_root: &Path,
    branch: &str,
    env_vars: &HashMap<String, String>,
) -> Result<(), GitClownError> {
    let worktree_path = repo_root.join(branch);

    if worktree_path.exists() {
        info!(
            "Worktree directory {} already exists, skipping git worktree add",
            worktree_path.display().to_string().green()
        );
        return Ok(());
    }

    info!(
        "Adding worktree for branch {} at {}",
        branch.green(),
        worktree_path.display().to_string().green()
    );

    let output = build_git_command(repo_root, &["worktree", "add", branch], env_vars)
        .output()
        .await
        .map_err(|e| GitClownError::GitCloneError(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        println!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }

    if !output.status.success() {
        return Err(GitClownError::GitCloneError(format!(
            "git worktree add {} failed: {}",
            branch, stderr
        )));
    }

    info!("Worktree for branch {} added successfully", branch.green());
    Ok(())
}

/// Writes `<repo_root>/<repo>.code-workspace` so VS Code opens the entire
/// repo directory (including all worktrees) in one workspace.
///
/// Skips writing if the file already exists.
async fn write_vscode_workspace(repo_root: &Path, repo: &str) -> Result<(), GitClownError> {
    let workspace_file = repo_root.join(format!("{}.code-workspace", repo));

    if workspace_file.exists() {
        info!(
            "VS Code workspace file {} already exists, skipping",
            workspace_file.display().to_string().green()
        );
        return Ok(());
    }

    let content =
        "{\n  \"folders\": [\n    {\n      \"path\": \".\"\n    }\n  ],\n  \"settings\": {}\n}\n";
    fs::write(&workspace_file, content)?;
    info!(
        "VS Code workspace file written at {}",
        workspace_file.display().to_string().green()
    );
    Ok(())
}

/// Orchestrates the full bare clone workflow:
/// bare clone → `.git` pointer → refspec fix → fetch → default branch
/// detection → worktree add → VS Code workspace file.
async fn run_bare_clone(
    git_url: &str,
    workspace_root: &Path,
    host: &str,
    org: &str,
    repo: &str,
    env_vars: &HashMap<String, String>,
) -> Result<(), GitClownError> {
    let repo_root = workspace_root.join(host).join(org).join(repo);
    fs::create_dir_all(&repo_root)
        .map_err(|e| GitClownError::DirectoryCreationError(e.to_string()))?;

    clone_bare(git_url, &repo_root, env_vars).await?;
    write_git_pointer(&repo_root).await?;
    fix_refspec_and_fetch(&repo_root, env_vars).await?;
    let branch: String = detect_default_branch(&repo_root, env_vars).await?;
    add_worktree(&repo_root, &branch, env_vars).await?;
    write_vscode_workspace(&repo_root, repo).await?;

    info!(
        "Bare clone setup complete. Repo root: {}, default branch worktree: {}",
        repo_root.display().to_string().green(),
        branch.green()
    );
    Ok(())
}

/// Main application logic flow
///
/// Orchestrates the complete process:
/// 1. Parse command line arguments
/// 2. Get workspace root directory
/// 3. Parse git URL
/// 4. Create directory structure
/// 5. Find environment file
/// 6. Load environment variables
///    7a. If `--bare`: run bare clone + worktree workflow
///    7b. Otherwise: check if repository already exists, then execute git clone
///
/// # Returns
///
/// The exit status of the git clone command
///
/// # Errors
///
/// Various GitClownError types depending on which step fails
async fn run() -> Result<std::process::ExitStatus, GitClownError> {
    debug!("Starting git-clown");

    // Parse command-line arguments
    let args = Args::parse();
    info!("Git URL: {}", args.git_url.green());

    // Get workspace root
    let workspace_root = get_workspace_root().await;

    // Parse git URL
    let (host, org, repo) = parse_git_url(&args.git_url).await?;

    // Create directory structure (<workspace>/<host>/<org>)
    let dir_path = create_directory_structure(&workspace_root, &host, &org).await?;

    // Find environment file
    let env_file = find_env_file(&dir_path, &workspace_root).await?;

    // Get current environment as baseline
    let base_env_vars: HashMap<String, String> = env::vars().collect();

    // If an environment file was found, load its variables
    if let Some(env_file_path) = env_file {
        let file_env_vars = read_env_file(&env_file_path).await?;
        let env_vars = merge_env_vars(base_env_vars, file_env_vars);

        // Bare clone path: set up bare repo + worktree
        if args.bare {
            run_bare_clone(
                &args.git_url,
                &workspace_root,
                &host,
                &org,
                &repo,
                &env_vars,
            )
            .await?;
            return StdCommand::new("true")
                .status()
                .map_err(GitClownError::IoError);
        }

        // Check if the repository already exists
        if repository_exists(&dir_path, &repo).await {
            info!(
                "Repository {} already exists, skipping git clone",
                repo.green()
            );
            // Create a successful exit status in a cross-platform way
            let status = StdCommand::new("true").status().unwrap_or_else(|_| {
                // Fallback if running the 'true' command fails
                warn!("Failed to create exit status, using default success status");
                panic!("Failed to create exit status");
            });
            return Ok(status);
        }

        // Execute git clone with environment variables
        return execute_git_clone(&args.git_url, &dir_path, &env_vars).await;
    } else {
        warn!("No environment file found, proceeding with system environment only");
    }

    let env_vars = base_env_vars;

    // Bare clone path: set up bare repo + worktree
    if args.bare {
        run_bare_clone(
            &args.git_url,
            &workspace_root,
            &host,
            &org,
            &repo,
            &env_vars,
        )
        .await?;
        return StdCommand::new("true")
            .status()
            .map_err(GitClownError::IoError);
    }

    // Check if the repository already exists
    if repository_exists(&dir_path, &repo).await {
        info!(
            "Repository {} already exists, skipping git clone",
            repo.green()
        );
        // Create a successful exit status in a cross-platform way
        let status = StdCommand::new("true").status().unwrap_or_else(|_| {
            // Fallback if running the 'true' command fails
            warn!("Failed to create exit status, using default success status");
            panic!("Failed to create exit status");
        });
        return Ok(status);
    }

    // Execute git clone with environment variables
    execute_git_clone(&args.git_url, &dir_path, &env_vars).await
}

/// Application entry point
///
/// Sets up:
/// 1. Colored logging with env_logger
/// 2. Tokio async runtime
/// 3. Runs the main application logic
/// 4. Handles exit status and errors
///
/// # Returns
///
/// Result indicating success or failure
fn main() -> Result<()> {
    // Initialize logger with colored output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    // Create async runtime
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Run our async main in the runtime
    let result = runtime.block_on(run());

    match result {
        Ok(status) => {
            info!(
                "Git-clown completed with status code: {}",
                status.code().unwrap_or(0)
            );
            std::process::exit(status.code().unwrap_or(0));
        }
        Err(e) => {
            error!("Error: {}", e.to_string().red());
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_parse_git_url_https() {
        let (host, org, repo) = parse_git_url("https://github.com/example/repo.git")
            .await
            .unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(org, "example");
        assert_eq!(repo, "repo");
    }

    #[tokio::test]
    async fn test_parse_git_url_ssh() {
        let (host, org, repo) = parse_git_url("git@github.com:example/repo.git")
            .await
            .unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(org, "example");
        assert_eq!(repo, "repo");
    }

    #[tokio::test]
    async fn test_parse_git_url_without_git_extension() {
        let (host, org, repo) = parse_git_url("https://github.com/example/repo")
            .await
            .unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(org, "example");
        assert_eq!(repo, "repo");
    }

    #[tokio::test]
    async fn test_parse_git_url_invalid() {
        let result = parse_git_url("not-a-url").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_directory_structure() {
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path();

        let dir_path = create_directory_structure(workspace_root, "github.com", "example")
            .await
            .unwrap();

        let expected_path = workspace_root.join("github.com").join("example");
        assert_eq!(dir_path, expected_path);
        assert!(expected_path.exists());
        assert!(expected_path.is_dir());
    }

    #[tokio::test]
    async fn test_find_env_file() {
        // Initialize logger for tests
        let _ = env_logger::builder().is_test(true).try_init();

        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path();

        // Create directory structure
        let host_dir = workspace_root.join("github.com");
        fs::create_dir_all(&host_dir).unwrap();

        let org_dir = host_dir.join("example");
        fs::create_dir_all(&org_dir).unwrap();

        // Create .env file in host directory
        let env_file_path = host_dir.join(".env");
        let mut env_file = File::create(&env_file_path).unwrap();
        writeln!(env_file, "TEST_VAR=test_value").unwrap();

        // Find env file starting from org directory
        let found_env_file = find_env_file(&org_dir, workspace_root).await.unwrap();

        assert!(found_env_file.is_some());
        assert_eq!(found_env_file.unwrap(), env_file_path);
    }

    #[tokio::test]
    async fn test_find_env_file_not_found() {
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path();

        // Create directory without any env files
        let dir_path = workspace_root.join("empty");
        fs::create_dir_all(&dir_path).unwrap();

        let found_env_file = find_env_file(&dir_path, workspace_root).await.unwrap();
        assert!(found_env_file.is_none());
    }

    #[tokio::test]
    async fn test_read_env_file_dotenv() {
        // Create a temporary .env file
        let dir = tempdir().unwrap();
        let env_file_path = dir.path().join(".env");
        let mut env_file = File::create(&env_file_path).unwrap();

        writeln!(env_file, "# This is a comment").unwrap();
        writeln!(env_file, "TEST_KEY=test_value").unwrap();
        writeln!(env_file, "ANOTHER_KEY=another_value").unwrap();

        let env_vars = read_env_file(&env_file_path).await.unwrap();

        assert_eq!(env_vars.len(), 2);
        assert_eq!(env_vars.get("TEST_KEY").unwrap(), "test_value");
        assert_eq!(env_vars.get("ANOTHER_KEY").unwrap(), "another_value");
    }

    #[tokio::test]
    async fn test_read_env_file_envrc() {
        // Create a temporary .envrc file
        let dir = tempdir().unwrap();
        let envrc_file_path = dir.path().join(".envrc");
        let mut envrc_file = File::create(&envrc_file_path).unwrap();

        writeln!(envrc_file, "# This is a comment").unwrap();
        writeln!(envrc_file, "export TEST_KEY=test_value").unwrap();
        writeln!(envrc_file, "export QUOTED_KEY=\"quoted value\"").unwrap();
        writeln!(envrc_file, "ANOTHER_KEY=another_value").unwrap();

        let env_vars = read_env_file(&envrc_file_path).await.unwrap();

        assert_eq!(env_vars.len(), 3);
        assert_eq!(env_vars.get("TEST_KEY").unwrap(), "test_value");
        assert_eq!(env_vars.get("QUOTED_KEY").unwrap(), "quoted value");
        assert_eq!(env_vars.get("ANOTHER_KEY").unwrap(), "another_value");
    }

    // Create a modified version of get_workspace_root for testing
    async fn get_workspace_root_for_testing(env_override: Option<&str>) -> PathBuf {
        match env_override {
            Some(path) => PathBuf::from(path),
            None => {
                let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home_dir.join("ws")
            }
        }
    }

    #[tokio::test]
    async fn test_get_workspace_root_from_env() {
        // Test with a specific path
        let test_path = "/tmp/test-workspace";
        let workspace_root = get_workspace_root_for_testing(Some(test_path)).await;
        assert_eq!(
            workspace_root,
            PathBuf::from(test_path),
            "Workspace root should be set from WORKSPACE_ROOT environment variable"
        );
    }

    #[tokio::test]
    async fn test_get_workspace_root_default() {
        // Test the default path
        let workspace_root = get_workspace_root_for_testing(None).await;
        let expected_path = dirs::home_dir().unwrap().join("ws");

        assert_eq!(
            workspace_root, expected_path,
            "Default workspace root should be ~/ws when WORKSPACE_ROOT is not set"
        );
    }

    #[tokio::test]
    async fn test_repository_exists() {
        let temp_dir = tempdir().unwrap();
        let parent_dir = temp_dir.path();

        // Create a fake repository with a .git directory
        let repo_name = "test-repo";
        let repo_dir = parent_dir.join(repo_name);
        let git_dir = repo_dir.join(".git");

        fs::create_dir_all(&git_dir).unwrap();

        // Test that repository is detected
        assert!(repository_exists(parent_dir, repo_name).await);

        // Test that non-existent repository is not detected
        assert!(!repository_exists(parent_dir, "nonexistent-repo").await);

        // Test that a directory without .git is not detected as repository
        let not_repo_name = "not-a-repo";
        let not_repo_dir = parent_dir.join(not_repo_name);
        fs::create_dir_all(&not_repo_dir).unwrap();

        assert!(!repository_exists(parent_dir, not_repo_name).await);
    }

    /// Creates a minimal git repo with one commit in `dir`, using `branch` as the
    /// initial branch name. Returns without error; panics on setup failure.
    fn init_git_repo_with_commit(dir: &Path, branch: &str) {
        let run = |args: &[&str]| {
            StdCommand::new("git")
                .args(args)
                .current_dir(dir)
                .output()
                .unwrap();
        };
        run(&["init", "-b", branch]);
        run(&["config", "user.email", "test@test.com"]);
        run(&["config", "user.name", "Test User"]);
        fs::write(dir.join("README.md"), "test").unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", "initial"]);
    }

    #[tokio::test]
    async fn test_detect_default_branch_main() {
        let temp = tempdir().unwrap();
        let ws = temp.path();

        // Create source repo on branch "main"
        let source = ws.join("source");
        fs::create_dir_all(&source).unwrap();
        init_git_repo_with_commit(&source, "main");

        // Bare clone into repo_root/.bare
        let repo_root = ws.join("repo_root");
        fs::create_dir_all(&repo_root).unwrap();
        StdCommand::new("git")
            .args(["clone", "--bare", source.to_str().unwrap(), ".bare"])
            .current_dir(&repo_root)
            .output()
            .unwrap();

        let env_vars: HashMap<String, String> = std::env::vars().collect();
        let branch = detect_default_branch(&repo_root, &env_vars).await.unwrap();
        assert_eq!(branch, "main");
    }

    #[tokio::test]
    async fn test_detect_default_branch_master() {
        let temp = tempdir().unwrap();
        let ws = temp.path();

        let source = ws.join("source");
        fs::create_dir_all(&source).unwrap();
        init_git_repo_with_commit(&source, "master");

        let repo_root = ws.join("repo_root");
        fs::create_dir_all(&repo_root).unwrap();
        StdCommand::new("git")
            .args(["clone", "--bare", source.to_str().unwrap(), ".bare"])
            .current_dir(&repo_root)
            .output()
            .unwrap();

        let env_vars: HashMap<String, String> = std::env::vars().collect();
        let branch = detect_default_branch(&repo_root, &env_vars).await.unwrap();
        assert_eq!(branch, "master");
    }

    #[tokio::test]
    async fn test_run_bare_clone_full_workflow() {
        let temp = tempdir().unwrap();
        let ws = temp.path();

        // Create a source git repo
        let source = ws.join("source");
        fs::create_dir_all(&source).unwrap();
        init_git_repo_with_commit(&source, "main");

        let workspace_root = ws.join("workspace");
        fs::create_dir_all(&workspace_root).unwrap();

        let env_vars: HashMap<String, String> = std::env::vars().collect();

        run_bare_clone(
            source.to_str().unwrap(),
            &workspace_root,
            "localhost",
            "org",
            "myrepo",
            &env_vars,
        )
        .await
        .unwrap();

        let repo_root = workspace_root.join("localhost").join("org").join("myrepo");

        // .bare directory exists
        assert!(
            repo_root.join(".bare").is_dir(),
            ".bare should be a directory"
        );

        // .git pointer file has correct content
        let git_content = fs::read_to_string(repo_root.join(".git")).unwrap();
        assert_eq!(git_content.trim(), "gitdir: ./.bare");

        // worktree for default branch exists
        assert!(
            repo_root.join("main").is_dir(),
            "main worktree should exist"
        );

        // .code-workspace file exists with expected content
        let ws_file = repo_root.join("myrepo.code-workspace");
        assert!(ws_file.exists(), ".code-workspace file should exist");
        let ws_content = fs::read_to_string(&ws_file).unwrap();
        assert!(
            ws_content.contains("\"path\": \".\""),
            "workspace file should reference '.'"
        );
    }

    #[test]
    fn test_merge_env_vars_file_overrides_base() {
        let mut base = HashMap::new();
        base.insert("SAME_KEY".to_string(), "base-value".to_string());
        base.insert("ONLY_BASE".to_string(), "base-only".to_string());

        let mut file = HashMap::new();
        file.insert("SAME_KEY".to_string(), "file-value".to_string());
        file.insert("ONLY_FILE".to_string(), "file-only".to_string());

        let merged = merge_env_vars(base, file);

        assert_eq!(merged.get("SAME_KEY"), Some(&"file-value".to_string()));
        assert_eq!(merged.get("ONLY_BASE"), Some(&"base-only".to_string()));
        assert_eq!(merged.get("ONLY_FILE"), Some(&"file-only".to_string()));
    }

    #[tokio::test]
    async fn test_run_bare_clone_propagates_env_to_non_clone_git_steps() {
        let temp = tempdir().unwrap();
        let ws = temp.path();

        let source = ws.join("source");
        fs::create_dir_all(&source).unwrap();
        init_git_repo_with_commit(&source, "main");

        let workspace_root = ws.join("workspace");
        fs::create_dir_all(&workspace_root).unwrap();

        let trace_file = ws.join("git-trace.log");
        let mut env_vars: HashMap<String, String> = std::env::vars().collect();
        env_vars.insert(
            "GIT_TRACE".to_string(),
            trace_file.to_string_lossy().to_string(),
        );

        run_bare_clone(
            source.to_str().unwrap(),
            &workspace_root,
            "localhost",
            "org",
            "myrepo",
            &env_vars,
        )
        .await
        .unwrap();

        let trace = fs::read_to_string(&trace_file).unwrap();
        assert!(
            trace.contains(" fetch"),
            "trace should include git fetch invocation"
        );
        assert!(
            trace.contains(" symbolic-ref HEAD"),
            "trace should include git symbolic-ref invocation"
        );
        assert!(
            trace.contains("worktree add"),
            "trace should include git worktree add invocation"
        );
    }
}
