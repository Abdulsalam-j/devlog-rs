mod config;
mod daily;
mod export;
mod git;
mod llm;
mod markdown;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "devlog", about = "Local-first developer log CLI")]
struct Cli {
    /// Optional path to a devlog.toml configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate today's devlog entry
    Daily,
    /// Export aggregated logs (stub)
    Export,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config)?;
    config.validate()?;
    let tz = config.timezone()?;

    match cli.command {
        Commands::Daily => daily::run(&config, tz)?,
        Commands::Export => export::run(&config, tz)?,
    }

    Ok(())
}
