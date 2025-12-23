use crate::config::Config;
use crate::markdown;
use anyhow::{Context, Result, bail};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use std::path::PathBuf;
use std::process::Command;

pub fn run(config: &Config, tz: Tz) -> Result<()> {
    if !config.export.enabled {
        bail!("Export disabled in configuration.");
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
        return Ok(());
    }

    let export_path = markdown::write_export(&config.daily.output_dir, start, end, &entries)?;
    let format = config.export.format.as_deref().unwrap_or("md");

    if format == "pdf" {
        if let Err(err) = render_pdf(&export_path) {
            eprintln!(
                "PDF export failed: {err}. Markdown export remains at {}.",
                export_path.display()
            );
        }
    }

    println!("Export written to {}", export_path.display());
    Ok(())
}

fn render_pdf(markdown_path: &PathBuf) -> Result<()> {
    let pdf_path = markdown_path.with_extension("pdf");
    let status = Command::new("pandoc")
        .arg(markdown_path.as_os_str())
        .arg("-o")
        .arg(&pdf_path)
        .status()
        .with_context(|| "failed to invoke pandoc")?;

    if !status.success() {
        bail!("pandoc exited with status {:?}", status.code());
    }

    println!("PDF written to {}", pdf_path.display());
    Ok(())
}
