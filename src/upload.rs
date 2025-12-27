use crate::config::Drive;
use anyhow::{bail, Context, Result};
use chrono::Datelike;
use std::path::PathBuf;
use std::process::Command;

/// Uploads a file to Google Drive using rclone.
/// File should already be named DevLog-YYYY.md
pub fn to_drive(config: &Drive, file_path: &PathBuf) -> Result<()> {
    let destination = format!("{}:{}", config.remote, config.folder);

    let filename = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("DevLog-unknown.md");

    println!("Uploading {} to {}...", file_path.display(), destination);

    let status = Command::new("rclone")
        .arg("copy")
        .arg(file_path.as_os_str())
        .arg(&destination)
        .arg("--progress")
        .status()
        .with_context(|| "failed to invoke rclone (is it installed and configured?)")?;

    if !status.success() {
        bail!(
            "rclone exited with status {:?}. Make sure rclone is configured with 'rclone config'.",
            status.code()
        );
    }

    println!("✅ Uploaded to Google Drive: {}/{}", destination, filename);
    Ok(())
}

