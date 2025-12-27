use crate::config::Daily;
use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

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

    if !config.overwrite_existing && entry_exists(&file_path, date)? {
        println!("Entry already exists for {}, skipping.", date);
        return Ok(file_path);
    }

    if config.overwrite_existing {
        remove_entry(&file_path, date)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .with_context(|| format!("failed to open {}", file_path.display()))?;

    if !file_exists {
        write_year_header(&mut file, date)?;
    } else {
        // Always add a blank line before new entry for proper spacing
        writeln!(file)?;
    }

    write_entry(&mut file, date, summary, commits)?;
    Ok(file_path)
}

fn entry_exists(path: &PathBuf, date: NaiveDate) -> Result<bool> {
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

fn remove_entry(path: &PathBuf, date: NaiveDate) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let mut contents = String::new();
    OpenOptions::new()
        .read(true)
        .open(path)
        .and_then(|mut f| f.read_to_string(&mut contents))
        .with_context(|| format!("failed to read {}", path.display()))?;

    let date_str = format!("{date}");
    // Try new format first: ## [[date]]
    let new_format = format!("## [[{date_str}]]");
    let old_format = format!("[[{date_str}]]");
    
    let start = contents.find(&new_format)
        .or_else(|| contents.find(&old_format));
    
    if let Some(start) = start {
        // Find the end: look for next entry (either format) or end of file
        let search_start = if contents[start..].starts_with("##") {
            start + new_format.len()
        } else {
            start + old_format.len()
        };
        
        let rest = &contents[search_start..];
        // Look for next entry in either format
        let next_new = rest.find("## [[");
        let next_old = rest.find("\n[[");
        let next_idx = match (next_new, next_old) {
            (Some(a), Some(b)) => Some(search_start + a.min(b)),
            (Some(a), None) => Some(search_start + a),
            (None, Some(b)) => Some(search_start + b),
            (None, None) => None,
        };
        
        let end = next_idx.unwrap_or_else(|| contents.len());
        
        // Remove the entry, preserving proper spacing
        // Remove from start of entry to end, but keep trailing newlines if next entry exists
        let remove_start = start;
        let remove_end = if next_idx.is_some() {
            // If there's a next entry, remove up to but keep one newline before it
            end
        } else {
            // If this is the last entry, remove everything including trailing newlines
            end
        };
        
        contents.replace_range(remove_start..remove_end, "");
        
        // Ensure file ends with proper spacing if we removed the last entry
        if next_idx.is_none() && !contents.ends_with('\n') && !contents.is_empty() {
            contents.push('\n');
        }

        // Rewrite file with truncated contents
        fs::write(path, contents)
            .with_context(|| format!("failed to rewrite {}", path.display()))?;
    }

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
