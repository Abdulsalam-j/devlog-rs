use crate::config::Git;
use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use std::process::Command;

pub fn fetch_commits(config: &Git, date: NaiveDate) -> Result<Vec<String>> {
    let repo = config
        .repo_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("git.repo_path is required"))?;

    let since = format!("{}T00:00:00", date);
    let until = format!("{}T23:59:59", date);

    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo);
    cmd.arg("log");
    cmd.arg("--all");
    cmd.arg(format!("--since={since}"));
    cmd.arg(format!("--until={until}"));
    if let Some(author) = &config.author {
        cmd.arg(format!("--author={author}"));
    }
    cmd.arg("--pretty=%s");

    let output = cmd.output().with_context(|| "failed to execute git log")?;
    if !output.status.success() {
        bail!(
            "git log failed with status {}",
            output.status.code().unwrap_or(-1)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<String> = stdout
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();

    Ok(commits)
}
