use crate::config::Daily;
use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn write_daily_entry(
    config: &Daily,
    date: NaiveDate,
    summary: &str,
    commits: &[String],
) -> Result<PathBuf> {
    let path = shellexpand::tilde(&config.output_dir).into_owned();
    let dir = PathBuf::from(&path);
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    let file_path = dir.join(format!("DevLog-{}.md", date.year()));
    let file_exists = file_path.exists();

    // If entry already exists, skip
    if entry_exists(&file_path, date)? {
        return Ok(PathBuf::new()); // Return empty path to indicate skip
    }

    // Check if we need a newline before writing the entry
    let needs_newline = if file_path.exists() {
        let contents = fs::read_to_string(&file_path)
            .with_context(|| format!("failed to read {}", file_path.display()))?;
        // Only add newline if file has content and doesn't end with newline
        !contents.is_empty() && !contents.ends_with('\n')
    } else {
        false
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .with_context(|| format!("failed to open {}", file_path.display()))?;

    if !file_exists {
        write_year_header(&mut file, date)?;
    } else if needs_newline {
        // Only add a blank line if needed for proper spacing (and not overwriting)
        writeln!(file)?;
    }

    write_entry(&mut file, date, summary, commits)?;
    Ok(file_path)
}

pub fn entry_exists(path: &std::path::Path, date: NaiveDate) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let mut contents = String::new();
    OpenOptions::new()
        .read(true)
        .open(path)
        .and_then(|mut f| f.read_to_string(&mut contents))
        .with_context(|| format!("failed to read {}", path.display()))?;
    // Check for both old format [[date]] and new format ## [[date]]
    let date_str = format!("{date}");
    Ok(contents.contains(&format!("[[{date_str}]]")))
}

fn write_year_header<W: Write>(mut writer: W, date: NaiveDate) -> Result<()> {
    writeln!(
        writer,
        r#"---
type: devlog
year: {year}
---

# Dev Log – {year}
"#,
        year = date.year(),
    )?;
    Ok(())
}

fn write_entry<W: Write>(
    mut writer: W,
    date: NaiveDate,
    summary: &str,
    commits: &[String],
) -> Result<()> {
    writeln!(writer, "## [[{date}]]\n")?;
    writeln!(writer, "🛠️ **Summary**\n{summary}\n")?;
    if commits.is_empty() {
        writeln!(writer, "🛑 **Commits**  \nNo commits for this day.\n")?;
    } else {
        writeln!(writer, "📦 **Commits**")?;
        for commit in commits {
            writeln!(writer, "- {commit}")?;
        }
        writeln!(writer)?;
    }
    Ok(())
}
