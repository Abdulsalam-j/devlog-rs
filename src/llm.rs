use crate::config::Llm;
use anyhow::Result;
use reqwest::blocking::Client;
use serde::Serialize;
use std::time::Duration;

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
            eprintln!("LLM summary failed: {err}. Falling back to simple summary.");
            Ok(default_summary(commits))
        }
    }
}

fn summarize(config: &Llm, commits: &[String]) -> Result<String> {
    let prompt = format!(
        "Summarize the following git commit titles into one concise sentence. \
Avoid buzzwords. Do not invent work. Use at most one emoji{}.\n\n{}",
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
    };

    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    let resp: OllamaResponse = client
        .post("http://localhost:11434/api/generate")
        .json(&body)
        .send()?
        .json()?;

    Ok(resp.response.trim().to_owned())
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
}

#[derive(serde::Deserialize)]
struct OllamaResponse {
    response: String,
}
