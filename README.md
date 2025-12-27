# 📝 DevLog

> **Automate your daily developer log** - Turn git commits into beautiful markdown logs with AI summaries, automatically backed up to Google Drive.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## ✨ Features

- 🤖 **AI-Powered Summaries** - Uses local Ollama to create natural summaries of your work
- 📅 **Daily Logging** - Automatically captures all commits from your git repository
- 📄 **Markdown Format** - Clean, readable logs in Obsidian-friendly format
- ☁️ **Auto Backup** - Uploads to Google Drive automatically via rclone
- 🔄 **Smart Overwrite** - Replace today's entry if you run it multiple times
- 📊 **Yearly Files** - Organized as `DevLog-YYYY.md` for easy navigation

## 🚀 Quick Start

### Installation

**Option 1: Quick Install Script (Recommended)**

```bash
# Clone the repository
git clone https://github.com/Abdulsalam-j/devlog-rs.git
cd devlog-rs

# Run install script
./install.sh

# Verify installation
devlog --help
```

**Option 2: Manual Install**

```bash
# Clone the repository
git clone https://github.com/Abdulsalam-j/devlog-rs.git
cd devlog-rs

# Build and install globally
cargo install --path .

# Verify installation
devlog --help
```

**That's it!** Now you can run `devlog log` from anywhere in your terminal.

> **Note:** Make sure `~/.cargo/bin` is in your PATH. If `devlog` command is not found, add this to your `~/.zshrc` or `~/.bashrc`:
> ```bash
> export PATH="$HOME/.cargo/bin:$PATH"
> ```

### First-Time Setup

1. **Create your config file:**

```bash
# Create config in your home directory (recommended)
mkdir -p ~/.config/devlog
cp devlog.toml ~/.config/devlog/devlog.toml
# Or create it in your project directory
cp devlog.toml ./devlog.toml
```

2. **Edit the config** (`~/.config/devlog/devlog.toml` or `./devlog.toml`):

```toml
[daily]
output_dir = "/Users/YOUR_USERNAME/Documents/Obsidian Vault"
overwrite_existing = true

[git]
repo_path = "/path/to/your/git/repo"
author = "your.email@example.com"

[llm]
enabled = true
model = "llama3"
use_emoji = true

[drive]
enabled = true
remote = "gdrive"
folder = "DevLog"
```

3. **Set up Google Drive (one-time):**

```bash
# Install rclone if you haven't
brew install rclone  # macOS
# or: sudo apt install rclone  # Linux

# Configure Google Drive
rclone config
# Follow prompts:
# - n (new remote)
# - Name: gdrive
# - Storage: 18 (Google Drive)
# - Scope: 1 (Full access)
# - Leave other options as default
# - y (use web browser)
# - Complete OAuth in browser
# - n (not a team drive)
# - y (confirm)
# - q (quit)
```

4. **Set up Ollama (optional, for AI summaries):**

```bash
# Install Ollama
brew install ollama  # macOS
# or: curl -fsSL https://ollama.com/install.sh | sh  # Linux

# Start Ollama service
ollama serve

# Pull the model (in another terminal)
ollama pull llama3
```

## 📖 Usage

### Basic Usage

```bash
# Log today's work
devlog log
```

That's it! The tool will:
1. ✅ Fetch today's commits from your git repo
2. 🤖 Generate an AI summary (if LLM enabled)
3. 📝 Append to `DevLog-YYYY.md` in your output directory
4. ☁️ Upload to Google Drive automatically

### What Gets Created

Your log file will look like this:

```markdown
## [[2025-12-27]]

🛠️ **Summary**
Today I implemented user authentication flow and refactored payment module. 🔒

📦 **Commits**
- Implement user authentication flow
- Refactor payment processing module
- Add unit tests for auth service
```

## ⚙️ Configuration

### Config File Locations

The tool looks for `devlog.toml` in this order:
1. `--config /path/to/devlog.toml` (if provided)
2. `./devlog.toml` (current directory)
3. `~/.config/devlog/devlog.toml` (home config)

### Configuration Options

#### `[daily]` - Daily Logging

```toml
[daily]
output_dir = "/path/to/your/notes"  # Where to save DevLog-YYYY.md files
overwrite_existing = true            # Replace today's entry if it exists
```

#### `[git]` - Git Repository

```toml
[git]
repo_path = "/path/to/your/repo"           # Your git repository path
author = "your.email@example.com"          # Filter commits by author email
```

#### `[llm]` - AI Summaries (Optional)

```toml
[llm]
enabled = true          # Enable AI summaries
model = "llama3"        # Ollama model name
use_emoji = true        # Include emoji in summaries
```

**Available Models:**
- `llama3` (recommended, ~4.7GB)
- `llama3.1`
- `mistral`
- `codellama` (for code-focused summaries)

#### `[drive]` - Google Drive Upload

```toml
[drive]
enabled = true          # Enable Google Drive upload
remote = "gdrive"       # rclone remote name (from rclone config)
folder = "DevLog"       # Folder name on Google Drive
```

## 🎯 Examples

### Example 1: Daily Work Log

```bash
# After a day of coding, just run:
devlog log

# Output:
# Daily log written.
# ✅ Uploaded to Google Drive: gdrive:DevLog/DevLog-2025.md
```

### Example 2: Multiple Runs Same Day

If you run `devlog log` multiple times in one day:
- First run: Creates entry
- Subsequent runs: Replaces today's entry (if `overwrite_existing = true`)

### Example 3: No Commits Today

```bash
devlog log

# Still creates entry:
# 🛠️ **Summary**
# No commits for this day.
```

## 🔧 Requirements

| Tool | Purpose | Installation |
|------|---------|--------------|
| **Rust** | Build the tool | [rustup.rs](https://rustup.rs) |
| **rclone** | Google Drive upload | `brew install rclone` |
| **Ollama** | AI summaries (optional) | `brew install ollama` |

## 🐛 Troubleshooting

### "rclone not found"
```bash
brew install rclone
rclone config  # Set up Google Drive
```

### "LLM summary failed"
- Make sure Ollama is running: `ollama serve`
- Check model is installed: `ollama list`
- Pull the model: `ollama pull llama3`

### "No commits for this day"
- Check `git.repo_path` is correct
- Verify `git.author` matches your commit email
- Make sure you have commits today: `git log --author="your@email.com" --since="today"`

### "Failed to write markdown entry"
- Check `daily.output_dir` path exists
- Ensure you have write permissions

### Config not found
- Create `devlog.toml` in current directory, or
- Create `~/.config/devlog/devlog.toml`

## 🛠️ Development

```bash
# Clone and build
git clone https://github.com/Abdulsalam-j/devlog-rs.git
cd devlog-rs
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## 📝 File Structure

```
DevLog-2025.md
├── ## [[2025-01-15]]
│   ├── 🛠️ **Summary** - AI-generated summary
│   └── 📦 **Commits** - List of all commits
├── ## [[2025-01-16]]
│   └── ...
└── ...
```

## 🤝 Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- AI summaries powered by [Ollama](https://ollama.com/)
- Google Drive sync via [rclone](https://rclone.org/)

---

**Made with ❤️ for developers who want to track their daily progress**
