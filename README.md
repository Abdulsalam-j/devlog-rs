# devlog-rs

A simple CLI to log your daily work from git commits, export to PDF, and upload to Google Drive.

## Quick Start

```bash
# 1. Build the tool
cargo build --release

# 2. Create config file (see Configuration below)
cp devlog.example.toml devlog.toml

# 3. Configure rclone for Google Drive
rclone config
# Choose: n (new remote) → name it "gdrive" → Google Drive → follow prompts

# 4. Run!
devlog log
```

## Commands

- `devlog log` – Generate today's log, export PDF, and upload to Google Drive
- `devlog --help` – Show help and getting started info

## Requirements

- **pandoc** – for PDF export (`brew install pandoc`)
- **rclone** – for Google Drive upload (`brew install rclone`)
- **ollama** – optional, for LLM summaries (`brew install ollama`)

## Configuration

The tool looks for `devlog.toml` in:
1. `--config /path/to/devlog.toml` if provided
2. `./devlog.toml`
3. `~/.config/devlog/devlog.toml`

### Example Config

```toml
[general]
timezone = "Asia/Amman"
working_days = ["Sun", "Mon", "Tue", "Wed", "Thu"]

[daily]
output_dir = "/path/to/your/notes"
overwrite_existing = true

[git]
repo_path = "/path/to/your/repo"
author = "your.email@example.com"

[llm]
enabled = true
model = "llama3"
use_emoji = true

[export]
enabled = true
frequency = "monthly"
format = "pdf"

[drive]
enabled = true
remote = "gdrive"      # rclone remote name
folder = "DevLog"      # folder on Google Drive
```

## What It Does

1. **Collects commits** – Reads today's git commits for the configured repo/author
2. **Summarizes** – Optionally uses local Ollama LLM to summarize commits
3. **Writes log** – Creates/updates yearly Markdown file (`2025.md`)
4. **Exports PDF** – Aggregates entries and exports to PDF via pandoc
5. **Uploads** – Uploads PDF to Google Drive via rclone

## Development

```bash
cargo fmt
cargo clippy
cargo build
```
