use anyhow::{Context, Result};
use serde::Deserialize;

use crate::models::OnlineSkillResult;

#[derive(Debug, Deserialize)]
struct SkillsShResponse {
    skills: Vec<SkillsShItem>,
}

#[derive(Debug, Deserialize)]
struct SkillsShItem {
    name: String,
    installs: u64,
    source: String,
}

pub fn search(query: &str, limit: usize) -> Result<Vec<OnlineSkillResult>> {
    let url = format!(
        "https://skills.sh/api/search?q={}&limit={}",
        urlencoding::encode(query),
        limit.clamp(1, 50)
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("skills-hub-rs")
        .build()?;
    let response = client
        .get(url)
        .send()
        .context("skills.sh search request failed")?
        .error_for_status()
        .context("skills.sh search returned an error")?;
    let body: SkillsShResponse = response.json().context("parse skills.sh response")?;
    Ok(body
        .skills
        .into_iter()
        .map(|item| OnlineSkillResult {
            source_url: format!("https://github.com/{}", item.source),
            name: item.name,
            installs: item.installs,
            source: item.source,
        })
        .collect())
}
