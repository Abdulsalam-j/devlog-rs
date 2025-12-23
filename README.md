# devlog-rs

Local-first Rust CLI to turn daily git commits into Obsidian-friendly Markdown logs.

## Commands

- `devlog daily` – collect today's commits and write a daily entry.
- `devlog export` – placeholder for future aggregation/export.

Pass `--config /path/to/devlog.toml` to override config lookup.

## Configuration

The tool looks for `devlog.toml` in:

1. `--config` path if provided
2. `./devlog.toml`
3. `~/.config/devlog/devlog.toml`

See `devlog.example.toml` for all fields and defaults.

## Behavior (MVP)

- Only runs on configured working days and after `daily.run_time` (local time).
- Reads commits via `git log` for the configured repo/author.
- Optional LLM summary via local Ollama HTTP API (`http://localhost:11434/api/generate`).
- Writes monthly Markdown files under `daily.output_dir/<year>/<year>-<month>.md`.
- Idempotent daily entries; set `overwrite_existing = true` to rewrite.
- Timezone-aware run gating and date stamping via `general.timezone` (required; falls back to `TZ` env if set).
- `devlog export` aggregates entries for the current week or month into Markdown; optional PDF via `pandoc` when `export.format = "pdf"`.

## Development

```bash
cargo fmt
cargo clippy
cargo test
```

