# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A speed-obsessed Rust binary that renders Claude Code's status line. Reads JSON from stdin, outputs colored status to stdout. Performance is the primary goal - every microsecond matters.

## Commands

```bash
just build      # Build release binary (optimized for size)
just test       # Run tests
just lint       # Run clippy with pedantic + -D warnings
just fmt        # Format code
just try        # Test with sample JSON input
just bench      # Benchmark with hyperfine (100 runs)
```

Always run `just fmt` before committing - CI enforces `cargo fmt --check`.

## Architecture

Single-file binary (`src/main.rs`) with these key functions:

- **Input parsing**: Deserialize JSON from Claude Code into `Input` struct
- **Git detection**: `find_git_repo_name()` and `find_git_branch()` read `.git` files directly (no spawning `git` commands - that would add ~8ms per call)
- **Worktree support**: Parses `.git` file's `gitdir:` pointer to find main repo name
- **Output**: Two lines - status info (colored) + working directory path (dimmed)

## Performance Design Decisions

1. **Zero external commands**: Read `.git/HEAD` directly (~5us) instead of `git rev-parse` (~8ms)
2. **Release profile**: `opt-level = "z"`, LTO, strip, `panic = "abort"` - optimized for size
3. **Clippy pedantic**: Enabled in `Cargo.toml` with specific cast lints allowed

## Testing Locally

```bash
# After building, test with actual git repo
echo '{"model":{"display_name":"Opus 4.5"},"cwd":"'(pwd)'"}' | ./target/release/statusline
```

## Releasing

```bash
git tag vX.Y.Z -m "Release description"
git push origin vX.Y.Z
```

GitHub Actions builds binaries for Linux, macOS (Intel + ARM), and Windows.
