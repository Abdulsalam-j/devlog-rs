use anyhow::{bail, Context, Result};
use chrono_tz::Tz;
use dirs::config_dir;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub daily: Daily,
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub llm: Llm,
    #[serde(default)]
    pub drive: Drive,
}

#[derive(Debug, Deserialize)]
pub struct Daily {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct Git {
    #[serde(default)]
    pub repo_path: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Llm {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_use_emoji")]
    pub use_emoji: bool,
}


#[derive(Debug, Deserialize)]
pub struct Drive {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_drive_remote")]
    pub remote: String,
    #[serde(default = "default_drive_folder")]
    pub folder: String,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        let candidate_paths = resolve_candidate_paths(path)?;
        for candidate in candidate_paths {
            if candidate.exists() {
                let contents = fs::read_to_string(&candidate)
                    .with_context(|| format!("failed to read config at {}", candidate.display()))?;
                let config: Config = toml::from_str(&contents)
                    .with_context(|| format!("failed to parse {}", candidate.display()))?;
                return Ok(config);
            }
        }

        bail!(
            "No configuration file found. Expected ./devlog.toml or ~/.config/devlog/devlog.toml"
        );
    }

    pub fn validate(&self) -> Result<()> {
        if self
            .git
            .repo_path
            .as_deref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
        {
            bail!("git.repo_path is required and cannot be empty");
        }

        validate_output_dir(&self.daily.output_dir)?;
        validate_repo_path(&self.git.repo_path)?;

        if self.llm.enabled && self.llm.model.trim().is_empty() {
            bail!("llm.model cannot be empty when llm.enabled = true");
        }

        Ok(())
    }

    /// Returns timezone from TZ env var, or defaults to UTC.
    pub fn timezone(&self) -> Result<Tz> {
        let tz_str = env::var("TZ").unwrap_or_else(|_| "UTC".into());
        tz_str.parse::<Tz>().with_context(|| {
            format!("invalid timezone: {tz_str} (use an IANA identifier like 'Asia/Amman')")
        })
    }
}

fn resolve_candidate_paths(explicit: Option<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if let Some(p) = explicit {
        paths.push(p);
    }
    paths.push(PathBuf::from("devlog.toml"));
    if let Some(mut dir) = config_dir() {
        dir.push("devlog");
        dir.push("devlog.toml");
        paths.push(dir);
    }
    Ok(paths)
}

fn default_output_dir() -> String {
    "./DevLog".into()
}

fn default_model() -> String {
    "llama3".into()
}

fn default_use_emoji() -> bool {
    true
}

impl Default for Daily {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
        }
    }
}

impl Default for Git {
    fn default() -> Self {
        Self {
            repo_path: None,
            author: None,
        }
    }
}

impl Default for Llm {
    fn default() -> Self {
        Self {
            enabled: false,
            model: default_model(),
            use_emoji: default_use_emoji(),
        }
    }
}

impl Default for Drive {
    fn default() -> Self {
        Self {
            enabled: false,
            remote: default_drive_remote(),
            folder: default_drive_folder(),
        }
    }
}

fn default_drive_remote() -> String {
    "gdrive".into()
}

fn default_drive_folder() -> String {
    "DevLog".into()
}

fn validate_output_dir(dir: &str) -> Result<()> {
    if dir.trim().is_empty() {
        bail!("daily.output_dir cannot be empty");
    }
    Ok(())
}

fn validate_repo_path(repo: &Option<String>) -> Result<()> {
    if let Some(path_str) = repo {
        let expanded = shellexpand::tilde(path_str).into_owned();
        let path = Path::new(&expanded);
        if !path.exists() {
            bail!("git.repo_path does not exist: {}", path.display());
        }
    }
    Ok(())
}
