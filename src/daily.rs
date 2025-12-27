use crate::config::Config;
use crate::{git, llm, markdown};
use anyhow::{Context, Result};
use chrono::Utc;
use chrono_tz::Tz;

pub fn run(config: &Config, tz: Tz) -> Result<()> {
    let today = Utc::now().with_timezone(&tz).date_naive();
    let commits = git::fetch_commits(&config.git, today)?;
    let summary = llm::summarize_if_enabled(&config.llm, &commits)?;

    markdown::write_daily_entry(&config.daily, today, &summary, &commits)
        .context("failed to write markdown entry")?;

    println!("Daily log written.");
    Ok(())
}
