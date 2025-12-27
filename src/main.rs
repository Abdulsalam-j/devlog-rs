mod config;
mod daily;
mod export;
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
exports to PDF, and uploads to Google Drive.

QUICK START:
  1. Create a devlog.toml config file (see README)
  2. Configure rclone with Google Drive: rclone config
  3. Run: devlog log

REQUIREMENTS:
  - pandoc (for PDF export)
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
    /// Generate today's devlog, export to PDF, and upload to Google Drive
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
            daily::run(&config, tz)?;

            // Step 2: Export to PDF
            if let Some(pdf_path) = export::run(&config, tz)? {
                // Step 3: Upload to Google Drive
                if config.drive.enabled {
                    upload::to_drive(&config.drive, &pdf_path)?;
                }
            }
        }
    }

    Ok(())
}
