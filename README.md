# claude-status-line

A fast Rust binary for rendering Claude Code's status line.

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
[Opus 4.5] $2 - 📂[my-project] - 25k/200k (13%) - 42us
```

| Field | Description |
|-------|-------------|
| `[Opus 4.5]` | Current model name |
| `$2` | Session cost (rounded to nearest dollar) |
| `📂[my-project]` | Current working directory basename |
| `25k/200k` | Context tokens used / total available |
| `(13%)` | Context usage percentage |
| `15us` | Status line render time (microseconds) |

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
