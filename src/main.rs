use serde::Deserialize;
use std::{
    io::{self, Read},
    path::Path,
    time::Instant,
};

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

fn dir_basename(cwd: &str) -> &str {
    Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("?")
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
        assert_eq!(dir_basename("/foo/bar/project"), "project");
        assert_eq!(dir_basename("/single"), "single");
        assert_eq!(dir_basename("relative/path"), "path");
    }

    #[test]
    fn dir_basename_handles_empty() {
        assert_eq!(dir_basename(""), "?");
    }
}

fn main() {
    let start = Instant::now();

    // Read JSON from stdin
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).ok();

    let input: Input = serde_json::from_str(&buf).unwrap_or_default();

    let (used_k, max_k, pct) = input.context_window.stats();
    let elapsed_us = start.elapsed().as_micros();

    println!(
        "[{}] ${} - 📂[{}] - {}k/{}k ({:.0}%) - {}us",
        input.model.name(),
        input.cost.rounded(),
        dir_basename(&input.cwd),
        used_k,
        max_k,
        pct,
        elapsed_us
    );
}
