use crate::config::Llm;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";

pub fn summarize_if_enabled(config: &Llm, commits: &[String]) -> Result<String> {
    if commits.is_empty() {
        return Ok("No commits for this day.".into());
    }

    if !config.enabled {
        return Ok(default_summary(commits));
    }

    match summarize(config, commits) {
        Ok(summary) => Ok(summary),
        Err(err) => {
            let hint = llm_error_hint(&err);
            eprintln!("LLM summary failed: {err}.{hint}");
            eprintln!("Falling back to simple summary.");
            Ok(default_summary(commits))
        }
    }
}

fn llm_error_hint(err: &anyhow::Error) -> String {
    let s = err.to_string();
    if s.contains("connection refused") || s.contains("Connection refused") {
        return " Make sure Ollama is running: run `ollama serve` in another terminal.".into();
    }
    if s.contains("timed out") || s.contains("Timeout") {
        return " Try increasing [llm] timeout_secs in devlog.toml (default 120).".into();
    }
    if s.contains("model") && (s.contains("not found") || s.contains("does not exist") || s.contains("404")) {
        return " Pull the model first: run `ollama pull <model>` (e.g. ollama pull llama3).".into();
    }
    String::new()
}

fn summarize(config: &Llm, commits: &[String]) -> Result<String> {
    let prompt = format!(
        "Summarize these commit messages into one concise sentence starting with 'Today I'. \
Only mention what is explicitly stated in the commits. Do not add details or context that is not in the commit messages. \
Be factual and direct. Use at most one emoji{}.\n\n{}",
        if config.use_emoji {
            " if appropriate"
        } else {
            ""
        },
        commits.join("\n")
    );

    let body = OllamaRequest {
        model: &config.model,
        prompt: &prompt,
        stream: false,
    };

    let timeout = Duration::from_secs(config.timeout_secs);
    let client = Client::builder().timeout(timeout).build()?;

    let resp: OllamaResponse = client
        .post(OLLAMA_URL)
        .json(&body)
        .send()
        .with_context(|| format!("Failed to reach Ollama at {OLLAMA_URL}. Is Ollama running?"))?
        .json()
        .with_context(|| "Invalid response from Ollama. Check that the model is pulled (e.g. ollama pull llama3).")?;

    if let Some(ref err_msg) = resp.error {
        anyhow::bail!("Ollama error: {err_msg}");
    }

    let raw = resp.response.as_deref().unwrap_or("").trim();
    let mut summary = raw.to_string();
    // Remove surrounding quotes
    while (summary.starts_with('"') && summary.ends_with('"')) || 
          (summary.starts_with('\'') && summary.ends_with('\'')) {
        summary = summary[1..summary.len()-1].trim().to_string();
    }
    // Also remove any trailing quotes that might be left
    summary = summary.trim_end_matches('"').trim_end_matches('\'').trim().to_string();

    if summary.is_empty() {
        return Ok(default_summary(commits));
    }
    Ok(summary)
}

fn default_summary(commits: &[String]) -> String {
    if commits.is_empty() {
        return "No commits for this day.".into();
    }
    format!("Worked on: {}", commits.join("; "))
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    #[serde(default)]
    response: Option<String>,
    #[serde(default)]
    error: Option<String>,
}
