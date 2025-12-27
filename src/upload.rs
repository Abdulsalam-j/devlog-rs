use crate::config::Drive;
use anyhow::{bail, Context, Result};
use chrono::Datelike;
use std::path::PathBuf;
use std::process::Command;

/// Uploads a file to Google Drive using rclone.
/// Renames the file to DevLog-YYYY.md format on Drive.
pub fn to_drive(config: &Drive, file_path: &PathBuf) -> Result<()> {
    let destination = format!("{}:{}", config.remote, config.folder);

    // Extract year from filename (e.g., "2025.md" -> "2025")
    let year = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or_else(|| chrono::Utc::now().year());

    let drive_filename = format!("DevLog-{}.md", year);
    let temp_path = file_path.parent().unwrap().join(&drive_filename);

    // Copy file with new name for upload
    std::fs::copy(file_path, &temp_path)
        .with_context(|| format!("failed to copy file to {}", temp_path.display()))?;

    println!("Uploading {} as {} to {}...", file_path.display(), drive_filename, destination);

    let status = Command::new("rclone")
        .arg("copy")
        .arg(&temp_path)
        .arg(&destination)
        .arg("--progress")
        .status()
        .with_context(|| "failed to invoke rclone (is it installed and configured?)")?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    if !status.success() {
        bail!(
            "rclone exited with status {:?}. Make sure rclone is configured with 'rclone config'.",
            status.code()
        );
    }

    println!("✅ Uploaded to Google Drive: {}/{}", destination, drive_filename);
    Ok(())
}

