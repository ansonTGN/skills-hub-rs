use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use tempfile::TempDir;
use uuid::Uuid;

use crate::fsops::{
    atomic_replace_dir, copy_dir, dir_hash, discover_skills, now_ms, paths_overlap,
    remove_path_any, run_git_clone, skill_metadata, sync_dir,
};
use crate::models::{
    CustomTool, DetectedSkill, InstallOptions, RecycleSnapshot, ResolvedTool, Scope, SkillRecord,
    SyncMode, TargetRecord,
};
use crate::store::Store;
use crate::tools::{
    builtin_tools, resolve_builtin, resolve_custom, resolve_target_root, validate_custom_key,
};

#[derive(Clone, Debug)]
pub struct SkillsHub {
    pub root: PathBuf,
    pub skills_dir: PathBuf,
    pub recycle_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub home: PathBuf,
    pub store: Store,
}

impl SkillsHub {
    pub fn open(root_override: Option<PathBuf>) -> Result<Self> {
        let home = dirs::home_dir().context("failed to resolve user home directory")?;
        let root = root_override
            .or_else(|| std::env::var_os("SKILLS_HUB_HOME").map(PathBuf::from))
            .unwrap_or_else(|| home.join(".skillshub-rs"));
        let skills_dir = root.join("skills");
        let recycle_dir = root.join("recycle");
        let temp_dir = root.join("tmp");
        fs::create_dir_all(&skills_dir)?;
        fs::create_dir_all(&recycle_dir)?;
        fs::create_dir_all(&temp_dir)?;
        let store = Store::new(root.join("skills-hub.sqlite3"))?;
        Ok(Self {
            root,
            skills_dir,
            recycle_dir,
            temp_dir,
            home,
            store,
        })
    }

    pub fn resolve_tool(&self, key: &str) -> Result<ResolvedTool> {
        if let Some(tool) = resolve_builtin(&self.home, key) {
            return Ok(tool);
        }
        let custom = self
            .store
            .get_custom_tool(key)?
            .with_context(|| format!("unknown tool {key:?}"))?;
        if !custom.enabled {
            bail!("custom tool {key} is disabled");
        }
        Ok(resolve_custom(&self.home, &custom))
    }

    pub fn list_tools(&self) -> Result<Vec<ResolvedTool>> {
        let mut out = Vec::new();
        for tool in builtin_tools() {
            if let Some(resolved) = resolve_builtin(&self.home, tool.key) {
                out.push(resolved);
            }
        }
        for tool in self.store.list_custom_tools()? {
            out.push(resolve_custom(&self.home, &tool));
        }
        Ok(out)
    }

    pub fn add_custom_tool(&self, mut tool: CustomTool) -> Result<()> {
        tool.key = tool.key.trim().to_string();
        tool.label = tool.label.trim().to_string();
        tool.global_dir = tool.global_dir.trim().to_string();
        validate_custom_key(&tool.key)?;
        if tool.label.is_empty() {
            bail!("custom tool label is required");
        }
        if tool.global_dir.is_empty() {
            bail!("custom tool global directory is required");
        }
        self.store.add_custom_tool(&tool)
    }

    pub fn remove_custom_tool(&self, key: &str) -> Result<()> {
        for skill in self.store.list_skills()? {
            for target in self.store.list_targets(&skill.id)? {
                if target.tool == key && target.status != "disabled" {
                    bail!(
                        "custom tool {key} still has active targets; unsync or disable them first"
                    );
                }
            }
        }
        self.store.remove_custom_tool(key)
    }

    pub fn install_local(&self, path: &Path, options: &InstallOptions) -> Result<Vec<SkillRecord>> {
        let canonical = fs::canonicalize(path)
            .with_context(|| format!("local source does not exist: {}", path.display()))?;
        if paths_overlap(&canonical, &self.skills_dir)? {
            bail!(
                "local source overlaps the managed central repository: {}",
                canonical.display()
            );
        }
        let discovered = discover_skills(&canonical, 4)?;
        if discovered.is_empty() {
            bail!("no SKILL.md found under {}", canonical.display());
        }
        let mut installed = Vec::new();
        for skill_dir in discovered {
            let record = self.install_one(
                &skill_dir,
                "local",
                Some(skill_dir.to_string_lossy().to_string()),
                None,
                None,
                options,
            )?;
            installed.push(record);
        }
        Ok(installed)
    }

    pub fn install_git(
        &self,
        url: &str,
        revision: Option<&str>,
        subdir: Option<&Path>,
        options: &InstallOptions,
    ) -> Result<Vec<SkillRecord>> {
        let temp = TempDir::new_in(&self.temp_dir)?;
        let clone_root = temp.path().join("repo");
        run_git_clone(url, revision, &clone_root)?;
        let search_root = subdir
            .map(|p| clone_root.join(p))
            .unwrap_or_else(|| clone_root.clone());
        if !search_root.is_dir() {
            bail!("git subdirectory does not exist: {}", search_root.display());
        }
        let discovered = discover_skills(&search_root, 4)?;
        if discovered.is_empty() {
            bail!("no SKILL.md found in repository {url}");
        }
        let mut installed = Vec::new();
        for skill_dir in discovered {
            let rel = skill_dir
                .strip_prefix(&clone_root)
                .unwrap_or(&skill_dir)
                .to_string_lossy()
                .replace('\\', "/");
            let record = self.install_one(
                &skill_dir,
                "git",
                Some(url.to_string()),
                if rel.is_empty() { None } else { Some(rel) },
                revision.map(str::to_string),
                options,
            )?;
            installed.push(record);
        }
        Ok(installed)
    }

    fn install_one(
        &self,
        source_dir: &Path,
        source_type: &str,
        source_ref: Option<String>,
        source_subdir: Option<String>,
        source_revision: Option<String>,
        options: &InstallOptions,
    ) -> Result<SkillRecord> {
        let (name, description) = skill_metadata(source_dir)?;
        if self.store.get_skill_by_name(&name)?.is_some() {
            bail!("skill {name:?} is already installed");
        }
        let central = self.skills_dir.join(&name);
        if fs::symlink_metadata(&central).is_ok() {
            bail!("managed target already exists: {}", central.display());
        }
        copy_dir(source_dir, &central)?;
        let now = now_ms();
        let skill = SkillRecord {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            source_type: source_type.to_string(),
            source_ref,
            source_subdir,
            source_revision,
            central_path: central.to_string_lossy().to_string(),
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        if let Err(err) = self.store.insert_skill(&skill) {
            let _ = remove_path_any(&central);
            return Err(err);
        }
        self.store.set_tags(&skill.id, &options.tags)?;

        for tool in &options.tools {
            if let Err(err) = self.sync_skill(
                &skill.name,
                tool,
                options.scope,
                options.project.as_deref(),
                options.mode,
                options.overwrite,
            ) {
                let failed = TargetRecord {
                    id: Uuid::new_v4().to_string(),
                    skill_id: skill.id.clone(),
                    tool: tool.clone(),
                    scope: options.scope.to_string(),
                    project_path: options
                        .project
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string()),
                    target_path: String::new(),
                    mode: options.mode.to_string(),
                    status: "error".to_string(),
                    last_error: Some(err.to_string()),
                    synced_at: None,
                };
                let _ = self.store.upsert_target(&failed);
            }
        }
        Ok(skill)
    }

    pub fn sync_skill(
        &self,
        skill_ref: &str,
        tool_key: &str,
        scope: Scope,
        project: Option<&Path>,
        requested_mode: SyncMode,
        overwrite: bool,
    ) -> Result<TargetRecord> {
        let skill = self
            .store
            .get_skill(skill_ref)?
            .with_context(|| format!("skill {skill_ref:?} not found"))?;
        if !skill.enabled {
            bail!("skill {} is disabled", skill.name);
        }
        let tool = self.resolve_tool(tool_key)?;
        let root = resolve_target_root(&tool, scope, project)?;
        let target = root.join(&skill.name);
        let central = PathBuf::from(&skill.central_path);
        if paths_overlap(&central, &target)? {
            bail!("sync target overlaps the central skill source");
        }
        if skill.source_type == "local" {
            if let Some(source) = skill.source_ref.as_deref() {
                if !source.trim().is_empty() && paths_overlap(Path::new(source), &target)? {
                    bail!("sync target overlaps the original local source: {source}");
                }
            }
        }

        let target_string = target.to_string_lossy().to_string();
        let project_string = project.map(|p| p.to_string_lossy().to_string());
        let existing = self.store.get_target(
            &skill.id,
            tool_key,
            &scope.to_string(),
            project_string.as_deref(),
        )?;
        let existing_same_path = self.store.targets_at_path(&target_string)?;
        if existing_same_path
            .iter()
            .any(|t| t.skill_id != skill.id && t.status != "disabled")
        {
            bail!(
                "target path is already owned by another managed skill: {}",
                target.display()
            );
        }
        // If this is a new logical relation but another tool already exposes the same Skill
        // through the exact same physical directory, reuse that filesystem target.
        if existing.is_none() {
            if let Some(shared_target) = existing_same_path
                .iter()
                .find(|t| t.skill_id == skill.id && t.status != "disabled")
            {
                let shared = TargetRecord {
                    id: Uuid::new_v4().to_string(),
                    skill_id: skill.id.clone(),
                    tool: tool_key.to_string(),
                    scope: scope.to_string(),
                    project_path: project_string.clone(),
                    target_path: target_string,
                    mode: shared_target.mode.clone(),
                    status: "ok".to_string(),
                    last_error: None,
                    synced_at: Some(now_ms()),
                };
                self.store.upsert_target(&shared)?;
                return Ok(shared);
            }
        }
        let mode = if requested_mode == SyncMode::Auto
            && !tool.builtin
            && tool.sync_mode != SyncMode::Auto
        {
            tool.sync_mode
        } else {
            requested_mode
        };
        let outcome = sync_dir(&central, &target, mode, tool.force_copy, overwrite)?;
        let record = TargetRecord {
            id: existing
                .map(|t| t.id)
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            skill_id: skill.id.clone(),
            tool: tool_key.to_string(),
            scope: scope.to_string(),
            project_path: project_string,
            target_path: outcome.target_path.to_string_lossy().to_string(),
            mode: outcome.mode_used.to_string(),
            status: "ok".to_string(),
            last_error: None,
            synced_at: Some(now_ms()),
        };
        self.store.upsert_target(&record)?;
        Ok(record)
    }

    pub fn unsync_skill(
        &self,
        skill_ref: &str,
        tool_key: &str,
        scope: Scope,
        project: Option<&Path>,
    ) -> Result<()> {
        let skill = self
            .store
            .get_skill(skill_ref)?
            .with_context(|| format!("skill {skill_ref:?} not found"))?;
        let project_string = project.map(|p| p.to_string_lossy().to_string());
        let target = self
            .store
            .get_target(
                &skill.id,
                tool_key,
                &scope.to_string(),
                project_string.as_deref(),
            )?
            .with_context(|| format!("no target for {} -> {}", skill.name, tool_key))?;
        if target.status != "disabled"
            && !target.target_path.is_empty()
            && self
                .store
                .target_path_reference_count(&target.target_path, Some(&target.id))?
                == 0
        {
            remove_path_any(Path::new(&target.target_path))?;
        }
        self.store.delete_target(&target.id)
    }

    pub fn disable_skill(&self, skill_ref: &str) -> Result<()> {
        let skill = self
            .store
            .get_skill(skill_ref)?
            .with_context(|| format!("skill {skill_ref:?} not found"))?;
        let targets = self.store.list_targets(&skill.id)?;
        for mut target in targets {
            if target.status != "disabled" && !target.target_path.is_empty() {
                if self
                    .store
                    .target_path_reference_count(&target.target_path, Some(&target.id))?
                    == 0
                {
                    remove_path_any(Path::new(&target.target_path))?;
                }
                target.status = "disabled".to_string();
                target.last_error = None;
                self.store.upsert_target(&target)?;
            }
        }
        self.store.set_skill_enabled(&skill.id, false, now_ms())
    }

    pub fn enable_skill(&self, skill_ref: &str) -> Result<()> {
        let skill = self
            .store
            .get_skill(skill_ref)?
            .with_context(|| format!("skill {skill_ref:?} not found"))?;
        self.store.set_skill_enabled(&skill.id, true, now_ms())?;
        let targets = self.store.list_targets(&skill.id)?;
        for target in targets {
            let scope: Scope = target.scope.parse()?;
            let mode: SyncMode = target.mode.parse().unwrap_or(SyncMode::Auto);
            let project = target.project_path.as_deref().map(Path::new);
            if let Err(err) = self.sync_skill(&skill.name, &target.tool, scope, project, mode, true)
            {
                let mut failed = target.clone();
                failed.status = "error".to_string();
                failed.last_error = Some(err.to_string());
                failed.synced_at = Some(now_ms());
                self.store.upsert_target(&failed)?;
            }
        }
        Ok(())
    }

    pub fn update_skill(&self, skill_ref: &str) -> Result<String> {
        let mut skill = self
            .store
            .get_skill(skill_ref)?
            .with_context(|| format!("skill {skill_ref:?} not found"))?;
        let central = PathBuf::from(&skill.central_path);
        let temp = TempDir::new_in(&self.temp_dir)?;
        let candidate = match skill.source_type.as_str() {
            "local" => {
                let source = skill
                    .source_ref
                    .as_deref()
                    .context("local skill has no source path")?;
                let path = PathBuf::from(source);
                if !path.join("SKILL.md").is_file() {
                    bail!("local source is missing or invalid: {}", path.display());
                }
                path
            }
            "git" => {
                let url = skill
                    .source_ref
                    .as_deref()
                    .context("git skill has no repository URL")?;
                let clone_root = temp.path().join("repo");
                run_git_clone(url, skill.source_revision.as_deref(), &clone_root)?;
                skill
                    .source_subdir
                    .as_deref()
                    .map(|p| clone_root.join(p))
                    .unwrap_or(clone_root)
            }
            other => bail!("skill source type {other:?} is not updateable"),
        };
        if !candidate.join("SKILL.md").is_file() {
            bail!(
                "updated source no longer contains SKILL.md: {}",
                candidate.display()
            );
        }
        let old_hash = dir_hash(&central)?;
        let new_hash = dir_hash(&candidate)?;
        if old_hash == new_hash {
            return Ok("up-to-date".to_string());
        }
        let (_, description) = skill_metadata(&candidate)?;
        atomic_replace_dir(&candidate, &central)?;
        skill.description = description;
        skill.updated_at = now_ms();
        self.store.update_skill(&skill)?;

        let targets = self.store.list_targets(&skill.id)?;
        for target in targets.into_iter().filter(|t| t.status != "disabled") {
            let scope: Scope = target.scope.parse()?;
            let mode: SyncMode = target.mode.parse().unwrap_or(SyncMode::Auto);
            let project = target.project_path.as_deref().map(Path::new);
            if let Err(err) = self.sync_skill(&skill.name, &target.tool, scope, project, mode, true)
            {
                let mut failed = target;
                failed.status = "error".to_string();
                failed.last_error = Some(err.to_string());
                failed.synced_at = Some(now_ms());
                self.store.upsert_target(&failed)?;
            }
        }
        Ok("updated".to_string())
    }

    pub fn update_all(&self) -> Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        for skill in self.store.list_skills()? {
            match self.update_skill(&skill.name) {
                Ok(status) => out.push((skill.name, status)),
                Err(err) => out.push((skill.name, format!("error: {err}"))),
            }
        }
        Ok(out)
    }

    pub fn remove_skill(&self, skill_ref: &str, permanent: bool) -> Result<Option<String>> {
        let skill = self
            .store
            .get_skill(skill_ref)?
            .with_context(|| format!("skill {skill_ref:?} not found"))?;
        let tags = self.store.tags_for_skill(&skill.id)?;
        let targets = self.store.list_targets(&skill.id)?;
        let central = PathBuf::from(&skill.central_path);

        let recycle_id = if permanent {
            None
        } else {
            let id = Uuid::new_v4().to_string();
            let backup = self.recycle_dir.join(&id).join("skill");
            if central.is_dir() {
                copy_dir(&central, &backup)?;
            }
            let snapshot = RecycleSnapshot {
                skill: skill.clone(),
                tags,
                targets: targets.clone(),
                deleted_at: now_ms(),
                backup_path: backup.to_string_lossy().to_string(),
            };
            let db_id = self.store.add_recycle_snapshot(&snapshot)?;
            Some((db_id, backup))
        };

        let mut unique_paths = std::collections::BTreeSet::new();
        for target in &targets {
            if target.status != "disabled" && !target.target_path.is_empty() {
                unique_paths.insert(target.target_path.clone());
            }
        }
        for target_path in unique_paths {
            let refs = self.store.targets_at_path(&target_path)?;
            let has_external_owner = refs
                .iter()
                .any(|t| t.skill_id != skill.id && t.status != "disabled");
            if !has_external_owner {
                let _ = remove_path_any(Path::new(&target_path));
            }
        }
        remove_path_any(&central)?;
        self.store.delete_skill_row(&skill.id)?;
        Ok(recycle_id.map(|(id, _)| id))
    }

    pub fn list_recycle(&self) -> Result<Vec<(String, RecycleSnapshot)>> {
        self.store.list_recycle()
    }

    pub fn restore_recycle(&self, recycle_id: &str) -> Result<SkillRecord> {
        let snapshot = self
            .store
            .get_recycle(recycle_id)?
            .with_context(|| format!("recycle entry {recycle_id:?} not found"))?;
        if self
            .store
            .get_skill_by_name(&snapshot.skill.name)?
            .is_some()
        {
            bail!(
                "cannot restore: skill name {} is already installed",
                snapshot.skill.name
            );
        }
        let backup = PathBuf::from(&snapshot.backup_path);
        if !backup.is_dir() {
            bail!("recycle backup is missing: {}", backup.display());
        }
        let central = PathBuf::from(&snapshot.skill.central_path);
        if fs::symlink_metadata(&central).is_ok() {
            bail!(
                "cannot restore: central path is occupied: {}",
                central.display()
            );
        }
        copy_dir(&backup, &central)?;
        let mut skill = snapshot.skill.clone();
        skill.updated_at = now_ms();
        if let Err(err) = self.store.insert_skill(&skill) {
            let _ = remove_path_any(&central);
            return Err(err);
        }
        self.store.set_tags(&skill.id, &snapshot.tags)?;

        for old in &snapshot.targets {
            if old.status == "disabled" {
                self.store.upsert_target(old)?;
                continue;
            }
            let scope: Scope = old.scope.parse()?;
            let mode: SyncMode = old.mode.parse().unwrap_or(SyncMode::Auto);
            let project = old.project_path.as_deref().map(Path::new);
            if let Err(err) = self.sync_skill(&skill.name, &old.tool, scope, project, mode, false) {
                let mut failed = old.clone();
                failed.status = "error".to_string();
                failed.last_error = Some(err.to_string());
                failed.synced_at = Some(now_ms());
                self.store.upsert_target(&failed)?;
            }
        }
        self.store.delete_recycle(recycle_id)?;
        if let Some(entry_root) = backup.parent() {
            let _ = remove_path_any(entry_root);
        }
        Ok(skill)
    }

    pub fn purge_recycle(&self, all: bool) -> Result<usize> {
        const THIRTY_DAYS_MS: i64 = 30 * 24 * 60 * 60 * 1000;
        let cutoff = now_ms() - THIRTY_DAYS_MS;
        let entries = self.store.list_recycle()?;
        let mut purged = 0;
        for (id, snapshot) in entries {
            if all || snapshot.deleted_at <= cutoff {
                let backup = PathBuf::from(&snapshot.backup_path);
                if let Some(entry_root) = backup.parent() {
                    let _ = remove_path_any(entry_root);
                }
                self.store.delete_recycle(&id)?;
                purged += 1;
            }
        }
        Ok(purged)
    }

    pub fn scan_tool(&self, tool_key: Option<&str>) -> Result<Vec<DetectedSkill>> {
        let tools = if let Some(key) = tool_key {
            vec![self.resolve_tool(key)?]
        } else {
            self.list_tools()?
                .into_iter()
                .filter(|tool| {
                    tool.detect_dir
                        .as_deref()
                        .map(|p| p.exists())
                        .unwrap_or(false)
                })
                .collect()
        };
        let mut out = Vec::new();
        for tool in tools {
            let root = tool.global_dir.clone();
            if !root.is_dir() {
                continue;
            }
            for path in discover_skills(&root, 4)? {
                let (name, _) = skill_metadata(&path)?;
                let is_link = fs::symlink_metadata(&path)
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false);
                out.push(DetectedSkill {
                    name,
                    path: path.to_string_lossy().to_string(),
                    tool: Some(tool.key.clone()),
                    is_link,
                });
            }
        }
        out.sort_by(|a, b| (&a.tool, &a.name, &a.path).cmp(&(&b.tool, &b.name, &b.path)));
        out.dedup_by(|a, b| a.path == b.path);
        Ok(out)
    }
}
