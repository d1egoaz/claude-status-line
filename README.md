# claude-status-line

A fast Rust binary for rendering Claude Code's status line with Tokyo Night colors.

![Screenshot](screenshot.png)

## Why?

Life's too short for slow status lines.

The bash version spawns 7 `jq` processes per render (121ms). This Rust version parses JSON once and gets out of your way (13ms). Same output, 9x faster, 100% less process spawning.

### Benchmark (hyperfine, 100 runs)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| Bash + jq (7 calls) | 121.1 ± 6.8 | 103.7 | 145.0 | 9.17 ± 1.46 |
| Rust binary | 13.2 ± 2.0 | 9.4 | 19.7 | 1.00 |

See `statusline.sh.example` for the original bash implementation.

## Build

Requires Rust. Install via [rustup](https://rustup.rs/) or your package manager.

```bash
# Build release binary
cargo build --release

# Or with just (https://github.com/casey/just)
just build
```

Binary outputs to `target/release/statusline`.

## Usage

Configure in `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/statusline",
    "padding": 0
  }
}
```

## Example Output

```
[Opus 4.5] $1 - [.claude] - 46k/200k (23%) - 27us
```

| Field | Color | Description |
|-------|-------|-------------|
| `[Opus 4.5]` | Blue | Current model name |
| `$1` | Green/Yellow/Orange | Session cost (green $0-5, yellow $6-20, orange $21+) |
| `[.claude]` | Purple | Current working directory basename |
| `46k/200k (23%)` | Cyan | Context tokens used / total available |
| `27us` | Gray | Status line render time (microseconds) |

Colors use the [Tokyo Night](https://github.com/enkia/tokyo-night-vscode-theme) palette.

## Development

```bash
just          # List available commands
just build    # Build release binary
just try      # Test with sample JSON
just lint     # Run clippy (requires clippy)
just fmt      # Format code
```

## License

MIT
