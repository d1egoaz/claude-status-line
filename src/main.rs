use serde::Deserialize;
use std::io;
use std::path::Path;
use std::time::Instant;

const DEFAULT_CONTEXT_WINDOW: u64 = 200_000;

#[derive(Deserialize, Default)]
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

#[derive(Deserialize, Default)]
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

#[derive(Deserialize, Default)]
struct Cost {
    #[serde(default)]
    total_cost_usd: f64,
}

#[derive(Deserialize, Default)]
struct ContextWindow {
    #[serde(default)]
    context_window_size: u64,
    #[serde(default)]
    used_percentage: f64,
}

impl ContextWindow {
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn stats(&self) -> (u64, u64, f64) {
        let max = if self.context_window_size > 0 {
            self.context_window_size
        } else {
            DEFAULT_CONTEXT_WINDOW
        };

        // Safe: token counts fit in f64, result is always positive, truncation is intentional
        let used_k = (max as f64 * self.used_percentage / 100.0 / 1000.0) as u64;
        let max_k = max / 1000;

        (used_k, max_k, self.used_percentage)
    }
}

fn main() {
    let start = Instant::now();

    // 1. Direct parsing from stdin is more memory efficient
    let input: Input = serde_json::from_reader(io::stdin()).unwrap_or_default();

    // 2. Use Path API for robust directory handling
    let dir_name = Path::new(&input.cwd)
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("?");

    // 3. Destructure stats for readability
    let (used_k, max_k, pct) = input.context_window.stats();

    // 4. Round cost to nearest dollar (safe: costs are small positive values)
    #[allow(clippy::cast_possible_truncation)]
    let cost = input.cost.total_cost_usd.round() as i64;

    let elapsed_us = start.elapsed().as_micros();

    println!(
        "[{}] ${} - 📂[{}] - {}k/{}k ({:.0}%) - {}us",
        input.model.name(),
        cost,
        dir_name,
        used_k,
        max_k,
        pct,
        elapsed_us
    );
}
