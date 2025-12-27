use crate::config::Config;
use crate::{git, llm, markdown};
use anyhow::{Context, Result};
use chrono::Utc;
use chrono_tz::Tz;
use std::path::PathBuf;

pub fn run(config: &Config, tz: Tz) -> Result<std::path::PathBuf> {
    let today = Utc::now().with_timezone(&tz).date_naive();
    let commits = git::fetch_commits(&config.git, today)?;
    let summary = llm::summarize_if_enabled(&config.llm, &commits)?;

    let file_path = markdown::write_daily_entry(&config.daily, today, &summary, &commits)
        .context("failed to write markdown entry")?;

    // Check if entry was actually written (not skipped)
    if file_path.as_os_str().is_empty() {
        println!("Entry already exists for {}, skipping.", today);
        return Ok(PathBuf::new()); // Return empty path to indicate skip
    }
    
    println!("Daily log written.");
    Ok(file_path)
}
