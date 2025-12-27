use crate::config::Daily;
use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, Write};
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

pub fn collect_entries_in_range(
    output_dir: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<(NaiveDate, String)>> {
    let root = PathBuf::from(shellexpand::tilde(output_dir).into_owned());
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    visit_md_files(&root, &mut |path| -> Result<()> {
        let file_entries = parse_file_entries(path, start, end)?;
        entries.extend(file_entries);
        Ok(())
    })?;

    entries.sort_by_key(|(date, _)| *date);
    Ok(entries)
}

pub fn write_export(
    output_dir: &str,
    start: NaiveDate,
    end: NaiveDate,
    entries: &[(NaiveDate, String)],
) -> Result<PathBuf> {
    let root = PathBuf::from(shellexpand::tilde(output_dir).into_owned());
    let export_dir = root.join("exports");
    fs::create_dir_all(&export_dir)
        .with_context(|| format!("failed to create {}", export_dir.display()))?;

    // Use year from the end date (current year)
    let year = end.year();
    let export_path = export_dir.join(format!("DevLog-{}.md", year));
    let mut file = File::create(&export_path)
        .with_context(|| format!("failed to create {}", export_path.display()))?;

    writeln!(file, "# Dev Log Export\n\nRange: {start} to {end}\n\n---\n")?;

    for (_, content) in entries {
        writeln!(file, "{content}\n")?;
    }

    Ok(export_path)
}

fn visit_md_files(dir: &Path, f: &mut dyn FnMut(&Path) -> Result<()>) -> Result<()> {
    if dir.is_file() {
        if dir.extension().and_then(|s| s.to_str()) == Some("md") {
            f(dir)?;
        }
        return Ok(());
    }

    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_md_files(&path, f)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            f(&path)?;
        }
    }
    Ok(())
}

fn parse_file_entries(
    path: &Path,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<(NaiveDate, String)>> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;

    let mut results = Vec::new();
    let mut current_date: Option<NaiveDate> = None;
    let mut buffer = String::new();

    for line in contents.lines() {
        if let Some(date) = parse_entry_header(line) {
            if let Some(d) = current_date {
                if is_in_range(d, start, end) {
                    results.push((d, buffer.clone()));
                }
            }
            current_date = Some(date);
            buffer.clear();
            buffer.push_str(line);
            buffer.push('\n');
        } else if current_date.is_some() {
            buffer.push_str(line);
            buffer.push('\n');
        }
    }

    if let Some(d) = current_date {
        if is_in_range(d, start, end) {
            results.push((d, buffer));
        }
    }

    Ok(results)
}

fn parse_entry_header(line: &str) -> Option<NaiveDate> {
    if let Some(rest) = line.strip_prefix("## [[") {
        if let Some(date_str) = rest.strip_suffix("]]") {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                return Some(date);
            }
        }
    }
    None
}

fn is_in_range(date: NaiveDate, start: NaiveDate, end: NaiveDate) -> bool {
    date >= start && date <= end
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
