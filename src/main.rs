use colored::Colorize;
use serde::Deserialize;
use std::{
    io::{self, Read},
    path::Path,
    time::Instant,
};

/// Tokyo Night color palette (RGB values)
mod tokyo {
    pub const BLUE: (u8, u8, u8) = (122, 162, 247); // #7aa2f7 - model
    pub const PURPLE: (u8, u8, u8) = (187, 154, 247); // #bb9af7 - folder
    pub const CYAN: (u8, u8, u8) = (125, 207, 255); // #7dcfff - usage
    pub const GREEN: (u8, u8, u8) = (158, 206, 106); // #9ece6a - price (low)
    pub const YELLOW: (u8, u8, u8) = (224, 175, 104); // #e0af68 - price (medium)
    pub const ORANGE: (u8, u8, u8) = (255, 158, 100); // #ff9e64 - price (high)
    pub const COMMENT: (u8, u8, u8) = (86, 95, 137); // #565f89 - time (dimmed)
}

/// Default context window size for Claude models (Opus 4.5, Sonnet 4, etc.)
const DEFAULT_CONTEXT_WINDOW: u64 = 200_000;

#[derive(Debug, Deserialize, Default)]
struct Input {
    #[serde(default)]
    model: Model,
    #[serde(default)]
    cost: Cost,
    #[serde(default)]
    cwd: String,
    #[serde(default)]
    context_window: ContextWindow,
}

#[derive(Debug, Deserialize, Default)]
struct Model {
    #[serde(default)]
    id: String,
    #[serde(default)]
    display_name: String,
}

impl Model {
    fn name(&self) -> &str {
        if !self.display_name.is_empty() {
            &self.display_name
        } else if !self.id.is_empty() {
            &self.id
        } else {
            "?"
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct Cost {
    #[serde(default)]
    total_cost_usd: f64,
}

impl Cost {
    fn rounded(&self) -> i64 {
        self.total_cost_usd.round() as i64
    }
}

#[derive(Debug, Deserialize, Default)]
struct ContextWindow {
    #[serde(default)]
    context_window_size: u64,
    #[serde(default)]
    used_percentage: f64,
}

impl ContextWindow {
    fn stats(&self) -> (u64, u64, f64) {
        let max = if self.context_window_size > 0 {
            self.context_window_size
        } else {
            DEFAULT_CONTEXT_WINDOW
        };

        let pct = self.used_percentage;

        // Round to nearest k
        let used_tokens = (max as f64 * (pct / 100.0)).round();
        let used_k = (used_tokens / 1000.0).round() as u64;
        let max_k = (max as f64 / 1000.0).round() as u64;

        (used_k, max_k, pct)
    }
}

fn dir_basename(cwd: &str) -> String {
    Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .map_or_else(|| "?".to_string(), String::from)
}

/// Returns the path to the git directory for this repo.
/// For worktrees, returns the worktree's gitdir path.
/// For normal repos, returns .git directory.
fn find_gitdir(cwd: &str) -> Option<std::path::PathBuf> {
    let git_path = Path::new(cwd).join(".git");

    if git_path.is_file() {
        // Worktree: .git is a file with "gitdir: /path/to/main/.git/worktrees/..."
        let content = std::fs::read_to_string(&git_path).ok()?;
        let gitdir = content.trim().strip_prefix("gitdir: ")?;
        Some(std::path::PathBuf::from(gitdir))
    } else if git_path.is_dir() {
        Some(git_path)
    } else {
        None
    }
}

/// Finds the git repository name by checking .git in the cwd.
/// For worktrees, parses the .git file to find the main repo.
/// Returns None if not in a git repo.
fn find_git_repo_name(cwd: &str) -> Option<String> {
    let git_path = Path::new(cwd).join(".git");

    if git_path.is_file() {
        // Worktree: .git is a file with "gitdir: /path/to/main/.git/worktrees/..."
        let content = std::fs::read_to_string(&git_path).ok()?;
        let gitdir = content.trim().strip_prefix("gitdir: ")?;

        // gitdir looks like: /path/to/main-repo/.git/worktrees/<name>
        // We want to extract "main-repo" from this path
        let path = Path::new(gitdir);

        // Navigate up: worktrees/<name> -> .git -> main-repo
        let main_git = path.parent()?.parent()?; // .git directory
        let main_repo = main_git.parent()?; // main repo directory

        main_repo
            .file_name()
            .and_then(|s| s.to_str())
            .map(String::from)
    } else if git_path.is_dir() {
        // Normal repo: .git is a directory, repo name is cwd's basename
        Path::new(cwd)
            .file_name()
            .and_then(|s| s.to_str())
            .map(String::from)
    } else {
        // Not a git repo
        None
    }
}

/// Gets the current git branch name.
/// Returns None if detached HEAD or not in a git repo.
fn find_git_branch(cwd: &str) -> Option<String> {
    let gitdir = find_gitdir(cwd)?;
    let head_path = gitdir.join("HEAD");
    let head = std::fs::read_to_string(head_path).ok()?;

    // Parse "ref: refs/heads/branch-name"
    head.trim()
        .strip_prefix("ref: refs/heads/")
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_name_prefers_display_name() {
        let m = Model {
            display_name: "Opus 4.5".into(),
            id: "claude-opus".into(),
        };
        assert_eq!(m.name(), "Opus 4.5");
    }

    #[test]
    fn model_name_falls_back_to_id() {
        let m = Model {
            display_name: String::new(),
            id: "claude-opus".into(),
        };
        assert_eq!(m.name(), "claude-opus");
    }

    #[test]
    fn model_name_fallback_to_question_mark() {
        let m = Model::default();
        assert_eq!(m.name(), "?");
    }

    #[test]
    fn context_stats_uses_provided_size() {
        let ctx = ContextWindow {
            context_window_size: 100_000,
            used_percentage: 50.0,
        };
        let (used_k, max_k, pct) = ctx.stats();
        assert_eq!(max_k, 100);
        assert_eq!(used_k, 50);
        assert!((pct - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn context_stats_uses_default_when_zero() {
        let ctx = ContextWindow {
            context_window_size: 0,
            used_percentage: 10.0,
        };
        let (_, max_k, _) = ctx.stats();
        assert_eq!(max_k, 200); // DEFAULT_CONTEXT_WINDOW / 1000
    }

    #[test]
    fn dir_basename_extracts_last_component() {
        assert_eq!(dir_basename("/foo/bar/project"), "project".to_string());
        assert_eq!(dir_basename("/single"), "single".to_string());
        assert_eq!(dir_basename("relative/path"), "path".to_string());
    }

    #[test]
    fn dir_basename_handles_empty() {
        assert_eq!(dir_basename(""), "?".to_string());
    }
}

/// Shortens a path by replacing the home directory with ~
fn shorten_path(path: &str) -> String {
    std::env::var("HOME")
        .ok()
        .filter(|home| path.starts_with(home))
        .map_or_else(
            || path.to_string(),
            |home| format!("~{}", &path[home.len()..]),
        )
}

/// Returns RGB color for price based on cost thresholds
fn price_color(cost: i64) -> (u8, u8, u8) {
    match cost {
        0..=5 => tokyo::GREEN,   // cheap
        6..=20 => tokyo::YELLOW, // moderate
        _ => tokyo::ORANGE,      // expensive
    }
}

fn main() {
    let start = Instant::now();

    // Force colors on even when stdout is piped (Claude Code captures via pipe)
    colored::control::set_override(true);

    // Read JSON from stdin
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).ok();

    let input: Input = serde_json::from_str(&buf).unwrap_or_default();

    let (used_k, max_k, pct) = input.context_window.stats();
    let elapsed_us = start.elapsed().as_micros();

    let cost = input.cost.rounded();
    let (pr, pg, pb) = price_color(cost);

    // Line 1: [Model] $cost - [repo:branch] - tokens - time
    let repo_name = find_git_repo_name(&input.cwd).unwrap_or_else(|| dir_basename(&input.cwd));
    let repo_display = match find_git_branch(&input.cwd) {
        Some(branch) => format!("{repo_name}:{branch}"),
        None => repo_name,
    };

    println!(
        "[{}] {} - [{}] - {} - {}",
        input
            .model
            .name()
            .truecolor(tokyo::BLUE.0, tokyo::BLUE.1, tokyo::BLUE.2),
        format!("${cost}").truecolor(pr, pg, pb),
        repo_display.truecolor(tokyo::PURPLE.0, tokyo::PURPLE.1, tokyo::PURPLE.2),
        format!("{used_k}k/{max_k}k ({pct:.0}%)").truecolor(
            tokyo::CYAN.0,
            tokyo::CYAN.1,
            tokyo::CYAN.2
        ),
        format!("{elapsed_us}us").truecolor(tokyo::COMMENT.0, tokyo::COMMENT.1, tokyo::COMMENT.2),
    );

    // Line 2: Working directory path with ~ for home (dimmed)
    println!(
        "{}",
        shorten_path(&input.cwd).truecolor(tokyo::COMMENT.0, tokyo::COMMENT.1, tokyo::COMMENT.2)
    );
}
