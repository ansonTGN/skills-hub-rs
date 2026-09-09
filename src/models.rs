use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub source_subdir: Option<String>,
    pub source_revision: Option<String>,
    pub central_path: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetRecord {
    pub id: String,
    pub skill_id: String,
    pub tool: String,
    pub scope: String,
    pub project_path: Option<String>,
    pub target_path: String,
    pub mode: String,
    pub status: String,
    pub last_error: Option<String>,
    pub synced_at: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomTool {
    pub key: String,
    pub label: String,
    pub global_dir: String,
    pub project_dir: Option<String>,
    pub detect_dir: Option<String>,
    pub sync_mode: SyncMode,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecycleSnapshot {
    pub skill: SkillRecord,
    pub tags: Vec<String>,
    pub targets: Vec<TargetRecord>,
    pub deleted_at: i64,
    pub backup_path: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Global,
    Project,
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Scope::Global => "global",
            Scope::Project => "project",
        })
    }
}

impl FromStr for Scope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "global" => Ok(Self::Global),
            "project" => Ok(Self::Project),
            _ => bail!("invalid scope {s:?}; expected global or project"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    #[default]
    Auto,
    Symlink,
    Junction,
    Copy,
}

impl Display for SyncMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SyncMode::Auto => "auto",
            SyncMode::Symlink => "symlink",
            SyncMode::Junction => "junction",
            SyncMode::Copy => "copy",
        })
    }
}

impl FromStr for SyncMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "symlink" => Ok(Self::Symlink),
            "junction" => Ok(Self::Junction),
            "copy" => Ok(Self::Copy),
            _ => bail!("invalid sync mode {s:?}; expected auto, symlink, junction, or copy"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToolAdapter {
    pub key: &'static str,
    pub label: &'static str,
    pub global_dir: &'static str,
    pub project_dir: Option<&'static str>,
    pub detect_dir: &'static str,
    pub force_copy: bool,
}

#[derive(Clone, Debug)]
pub struct ResolvedTool {
    pub key: String,
    pub label: String,
    pub global_dir: PathBuf,
    pub project_dir: Option<PathBuf>,
    pub detect_dir: Option<PathBuf>,
    pub sync_mode: SyncMode,
    pub force_copy: bool,
    pub builtin: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectedSkill {
    pub name: String,
    pub path: String,
    pub tool: Option<String>,
    pub is_link: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnlineSkillResult {
    pub name: String,
    pub installs: u64,
    pub source: String,
    pub source_url: String,
}

#[derive(Clone, Debug)]
pub struct SyncOutcome {
    pub mode_used: SyncMode,
    pub target_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct InstallOptions {
    pub tags: Vec<String>,
    pub tools: Vec<String>,
    pub scope: Scope,
    pub project: Option<PathBuf>,
    pub mode: SyncMode,
    pub overwrite: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            tools: Vec::new(),
            scope: Scope::Global,
            project: None,
            mode: SyncMode::Auto,
            overwrite: false,
        }
    }
}
