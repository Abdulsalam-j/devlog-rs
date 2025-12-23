use crate::config::{Config, parse_time};
use crate::{git, llm, markdown};
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveTime, Utc, Weekday};
use chrono_tz::Tz;

pub fn run(config: &Config, tz: Tz) -> Result<()> {
    let now: DateTime<Tz> = Utc::now().with_timezone(&tz);
    ensure_working_day(config, now.weekday())?;
    ensure_run_time(config, now.time())?;

    let today = now.date_naive();
    let commits = git::fetch_commits(&config.git, today)?;
    let summary = llm::summarize_if_enabled(&config.llm, &commits)?;

    markdown::write_daily_entry(&config.daily, today, &summary, &commits)
        .context("failed to write markdown entry")?;

    println!("Daily log written.");
    Ok(())
}

fn ensure_working_day(config: &Config, today: Weekday) -> Result<()> {
    let today_str = weekday_to_str(today);
    if config
        .general
        .working_days
        .iter()
        .any(|day| day == &today_str)
    {
        return Ok(());
    }
    anyhow::bail!("Today ({today_str}) is not a configured working day.");
}

fn ensure_run_time(config: &Config, now: NaiveTime) -> Result<()> {
    let run_time = parse_time(&config.daily.run_time)?;
    if now < run_time {
        anyhow::bail!("Not time to run yet (configured run_time {}).", run_time);
    }
    Ok(())
}

fn weekday_to_str(day: Weekday) -> String {
    match day {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
    .into()
}
