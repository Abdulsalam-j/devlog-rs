use crate::config::Config;
use crate::markdown;
use anyhow::{bail, Context, Result};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use std::path::PathBuf;

/// Exports devlog entries and returns the export path for upload.
pub fn run(config: &Config, tz: Tz) -> Result<Option<PathBuf>> {
    if !config.export.enabled {
        println!("Export disabled in configuration, skipping.");
        return Ok(None);
    }

    let today = Utc::now().with_timezone(&tz).date_naive();
    let frequency = config.export.frequency.as_deref().unwrap_or("monthly");
    let (start, end) = match frequency {
        "weekly" => (today - Duration::days(6), today),
        "monthly" => (
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .context("failed to compute start of month")?,
            today,
        ),
        other => bail!("Unsupported export.frequency: {other}"),
    };

    let entries = markdown::collect_entries_in_range(&config.daily.output_dir, start, end)?;
    if entries.is_empty() {
        println!("No entries found between {start} and {end}; nothing to export.");
        return Ok(None);
    }

    let export_path = markdown::write_export(&config.daily.output_dir, start, end, &entries)?;
    println!("Export written to {}", export_path.display());
    Ok(Some(export_path))
}
