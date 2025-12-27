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
    pub general: General,
    #[serde(default)]
    pub daily: Daily,
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub llm: Llm,
    #[serde(default)]
    pub export: Export,
    #[serde(default)]
    pub drive: Drive,
}

#[derive(Debug, Deserialize)]
pub struct General {
    #[serde(default)]
    #[allow(dead_code)]
    pub timezone: Option<String>,
    #[serde(default = "default_working_days")]
    pub working_days: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Daily {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default)]
    pub overwrite_existing: bool,
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
pub struct Export {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub frequency: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
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

        validate_working_days(&self.general.working_days)?;
        validate_output_dir(&self.daily.output_dir)?;
        validate_repo_path(&self.git.repo_path)?;

        if self.llm.enabled && self.llm.model.trim().is_empty() {
            bail!("llm.model cannot be empty when llm.enabled = true");
        }

        if self.export.enabled {
            validate_export(&self.export)?;
        }

        self.timezone()?; // validate timezone string if provided
        Ok(())
    }

    pub fn timezone(&self) -> Result<Tz> {
        let tz_str = if let Some(tz) = &self.general.timezone {
            tz.clone()
        } else if let Ok(env_tz) = env::var("TZ") {
            env_tz
        } else {
            bail!(
                "general.timezone is required. Set it to an IANA timezone like 'Asia/Amman' \
or set the TZ environment variable."
            );
        };

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

fn default_working_days() -> Vec<String> {
    vec![
        "Mon".into(),
        "Tue".into(),
        "Wed".into(),
        "Thu".into(),
        "Fri".into(),
    ]
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

impl Default for General {
    fn default() -> Self {
        Self {
            timezone: None,
            working_days: default_working_days(),
        }
    }
}

impl Default for Daily {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            overwrite_existing: false,
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

impl Default for Export {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: Some(default_export_frequency()),
            format: Some(default_export_format()),
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

fn validate_export(export: &Export) -> Result<()> {
    let frequency = export
        .frequency
        .as_deref()
        .unwrap_or(DEFAULT_EXPORT_FREQUENCY);
    let format = export.format.as_deref().unwrap_or(DEFAULT_EXPORT_FORMAT);

    match frequency {
        "weekly" | "monthly" => {}
        other => bail!("export.frequency must be 'weekly' or 'monthly', got {other}"),
    }

    match format {
        "md" | "pdf" => {}
        other => bail!("export.format must be 'md' or 'pdf', got {other}"),
    }

    Ok(())
}

fn default_export_frequency() -> String {
    DEFAULT_EXPORT_FREQUENCY.to_string()
}

fn default_export_format() -> String {
    DEFAULT_EXPORT_FORMAT.to_string()
}

const DEFAULT_EXPORT_FREQUENCY: &str = "monthly";
const DEFAULT_EXPORT_FORMAT: &str = "md";

fn validate_working_days(days: &[String]) -> Result<()> {
    for day in days {
        match day.as_str() {
            "Mon" | "Tue" | "Wed" | "Thu" | "Fri" | "Sat" | "Sun" => {}
            other => bail!("general.working_days contains invalid value: {other}"),
        }
    }
    Ok(())
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
