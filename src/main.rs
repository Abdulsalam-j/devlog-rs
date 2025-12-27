mod config;
mod daily;
mod git;
mod llm;
mod markdown;
mod upload;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "devlog",
    about = "A simple CLI to log your daily work and upload to Google Drive",
    long_about = "DevLog - Local-first developer log CLI

Generates a daily work log from your git commits, summarizes them using an LLM,
and uploads to Google Drive.

QUICK START:
  1. Create a devlog.toml config file (see README)
  2. Configure rclone with Google Drive: rclone config
  3. Run: devlog log

REQUIREMENTS:
  - rclone (for Google Drive upload)
  - ollama (optional, for LLM summaries)"
)]
struct Cli {
    /// Path to a devlog.toml configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate today's devlog and upload to Google Drive
    Log,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config)?;
    config.validate()?;
    let tz = config.timezone()?;

    match cli.command {
        Commands::Log => {
            // Step 1: Generate today's entry
            let yearly_file = daily::run(&config, tz)?;

            // Step 2: Upload the yearly file to Google Drive (only if entry was written)
            if config.drive.enabled && !yearly_file.as_os_str().is_empty() {
                upload::to_drive(&config.drive, yearly_file.as_path())?;
            }
        }
    }

    Ok(())
}
