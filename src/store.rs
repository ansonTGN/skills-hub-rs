use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{CustomTool, RecycleSnapshot, SkillRecord, SyncMode, TargetRecord};

#[derive(Clone, Debug)]
pub struct Store {
    db_path: PathBuf,
}

impl Store {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let store = Self { db_path };
        store.init()?;
        Ok(store)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    fn open(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)
            .with_context(|| format!("open SQLite database {}", self.db_path.display()))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(conn)
    }

    fn init(&self) -> Result<()> {
        let conn = self.open()?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                source_type TEXT NOT NULL,
                source_ref TEXT,
                source_subdir TEXT,
                source_revision TEXT,
                central_path TEXT NOT NULL UNIQUE,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS tags (
                name TEXT PRIMARY KEY
            );
            CREATE TABLE IF NOT EXISTS skill_tags (
                skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
                tag TEXT NOT NULL REFERENCES tags(name) ON DELETE CASCADE,
                PRIMARY KEY (skill_id, tag)
            );
            CREATE TABLE IF NOT EXISTS targets (
                id TEXT PRIMARY KEY,
                skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
                tool TEXT NOT NULL,
                scope TEXT NOT NULL,
                project_path TEXT NOT NULL DEFAULT '',
                target_path TEXT NOT NULL,
                mode TEXT NOT NULL,
                status TEXT NOT NULL,
                last_error TEXT,
                synced_at INTEGER,
                UNIQUE(skill_id, tool, scope, project_path)
            );
            CREATE INDEX IF NOT EXISTS idx_targets_path ON targets(target_path);
            CREATE TABLE IF NOT EXISTS custom_tools (
                key TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                global_dir TEXT NOT NULL,
                project_dir TEXT,
                detect_dir TEXT,
                sync_mode TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS recycle (
                id TEXT PRIMARY KEY,
                snapshot_json TEXT NOT NULL,
                deleted_at INTEGER NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn insert_skill(&self, skill: &SkillRecord) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            r#"INSERT INTO skills
               (id,name,description,source_type,source_ref,source_subdir,source_revision,central_path,enabled,created_at,updated_at)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)"#,
            params![
                skill.id,
                skill.name,
                skill.description,
                skill.source_type,
                skill.source_ref,
                skill.source_subdir,
                skill.source_revision,
                skill.central_path,
                skill.enabled as i64,
                skill.created_at,
                skill.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_skill(&self, skill: &SkillRecord) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            r#"UPDATE skills SET
               name=?2, description=?3, source_type=?4, source_ref=?5, source_subdir=?6,
               source_revision=?7, central_path=?8, enabled=?9, updated_at=?10
               WHERE id=?1"#,
            params![
                skill.id,
                skill.name,
                skill.description,
                skill.source_type,
                skill.source_ref,
                skill.source_subdir,
                skill.source_revision,
                skill.central_path,
                skill.enabled as i64,
                skill.updated_at,
            ],
        )?;
        Ok(())
    }

    fn row_to_skill(row: &rusqlite::Row<'_>) -> rusqlite::Result<SkillRecord> {
        Ok(SkillRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            source_type: row.get(3)?,
            source_ref: row.get(4)?,
            source_subdir: row.get(5)?,
            source_revision: row.get(6)?,
            central_path: row.get(7)?,
            enabled: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }

    pub fn get_skill(&self, id_or_name: &str) -> Result<Option<SkillRecord>> {
        let conn = self.open()?;
        conn.query_row(
            r#"SELECT id,name,description,source_type,source_ref,source_subdir,source_revision,
               central_path,enabled,created_at,updated_at FROM skills
               WHERE id=?1 OR name=?1 LIMIT 1"#,
            [id_or_name],
            Self::row_to_skill,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn get_skill_by_name(&self, name: &str) -> Result<Option<SkillRecord>> {
        let conn = self.open()?;
        conn.query_row(
            r#"SELECT id,name,description,source_type,source_ref,source_subdir,source_revision,
               central_path,enabled,created_at,updated_at FROM skills WHERE name=?1 LIMIT 1"#,
            [name],
            Self::row_to_skill,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_skills(&self) -> Result<Vec<SkillRecord>> {
        let conn = self.open()?;
        let mut stmt = conn.prepare(
            r#"SELECT id,name,description,source_type,source_ref,source_subdir,source_revision,
               central_path,enabled,created_at,updated_at FROM skills ORDER BY name"#,
        )?;
        let rows = stmt.query_map([], Self::row_to_skill)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn delete_skill_row(&self, skill_id: &str) -> Result<()> {
        let conn = self.open()?;
        conn.execute("DELETE FROM skills WHERE id=?1", [skill_id])?;
        Ok(())
    }

    pub fn set_skill_enabled(&self, skill_id: &str, enabled: bool, updated_at: i64) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            "UPDATE skills SET enabled=?2, updated_at=?3 WHERE id=?1",
            params![skill_id, enabled as i64, updated_at],
        )?;
        Ok(())
    }

    pub fn set_tags(&self, skill_id: &str, tags: &[String]) -> Result<()> {
        let mut conn = self.open()?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM skill_tags WHERE skill_id=?1", [skill_id])?;
        for tag in tags {
            let tag = tag.trim();
            if tag.is_empty() {
                continue;
            }
            tx.execute("INSERT OR IGNORE INTO tags(name) VALUES (?1)", [tag])?;
            tx.execute(
                "INSERT OR IGNORE INTO skill_tags(skill_id,tag) VALUES (?1,?2)",
                params![skill_id, tag],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn add_tag(&self, skill_id: &str, tag: &str) -> Result<()> {
        let tag = tag.trim();
        if tag.is_empty() {
            return Ok(());
        }
        let mut conn = self.open()?;
        let tx = conn.transaction()?;
        tx.execute("INSERT OR IGNORE INTO tags(name) VALUES (?1)", [tag])?;
        tx.execute(
            "INSERT OR IGNORE INTO skill_tags(skill_id,tag) VALUES (?1,?2)",
            params![skill_id, tag],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn remove_tag(&self, skill_id: &str, tag: &str) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            "DELETE FROM skill_tags WHERE skill_id=?1 AND tag=?2",
            params![skill_id, tag],
        )?;
        Ok(())
    }

    pub fn tags_for_skill(&self, skill_id: &str) -> Result<Vec<String>> {
        let conn = self.open()?;
        let mut stmt = conn.prepare("SELECT tag FROM skill_tags WHERE skill_id=?1 ORDER BY tag")?;
        let rows = stmt.query_map([skill_id], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn row_to_target(row: &rusqlite::Row<'_>) -> rusqlite::Result<TargetRecord> {
        let project: String = row.get(4)?;
        Ok(TargetRecord {
            id: row.get(0)?,
            skill_id: row.get(1)?,
            tool: row.get(2)?,
            scope: row.get(3)?,
            project_path: if project.is_empty() {
                None
            } else {
                Some(project)
            },
            target_path: row.get(5)?,
            mode: row.get(6)?,
            status: row.get(7)?,
            last_error: row.get(8)?,
            synced_at: row.get(9)?,
        })
    }

    pub fn upsert_target(&self, target: &TargetRecord) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            r#"INSERT INTO targets
               (id,skill_id,tool,scope,project_path,target_path,mode,status,last_error,synced_at)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
               ON CONFLICT(skill_id,tool,scope,project_path) DO UPDATE SET
                 target_path=excluded.target_path,
                 mode=excluded.mode,
                 status=excluded.status,
                 last_error=excluded.last_error,
                 synced_at=excluded.synced_at"#,
            params![
                target.id,
                target.skill_id,
                target.tool,
                target.scope,
                target.project_path.clone().unwrap_or_default(),
                target.target_path,
                target.mode,
                target.status,
                target.last_error,
                target.synced_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_targets(&self, skill_id: &str) -> Result<Vec<TargetRecord>> {
        let conn = self.open()?;
        let mut stmt = conn.prepare(
            r#"SELECT id,skill_id,tool,scope,project_path,target_path,mode,status,last_error,synced_at
               FROM targets WHERE skill_id=?1 ORDER BY tool,scope,project_path"#,
        )?;
        let rows = stmt.query_map([skill_id], Self::row_to_target)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_target(
        &self,
        skill_id: &str,
        tool: &str,
        scope: &str,
        project: Option<&str>,
    ) -> Result<Option<TargetRecord>> {
        let conn = self.open()?;
        conn.query_row(
            r#"SELECT id,skill_id,tool,scope,project_path,target_path,mode,status,last_error,synced_at
               FROM targets WHERE skill_id=?1 AND tool=?2 AND scope=?3 AND project_path=?4 LIMIT 1"#,
            params![skill_id, tool, scope, project.unwrap_or_default()],
            Self::row_to_target,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn delete_target(&self, target_id: &str) -> Result<()> {
        let conn = self.open()?;
        conn.execute("DELETE FROM targets WHERE id=?1", [target_id])?;
        Ok(())
    }

    pub fn targets_at_path(&self, target_path: &str) -> Result<Vec<TargetRecord>> {
        let conn = self.open()?;
        let mut stmt = conn.prepare(
            r#"SELECT id,skill_id,tool,scope,project_path,target_path,mode,status,last_error,synced_at
               FROM targets WHERE target_path=?1 ORDER BY tool"#,
        )?;
        let rows = stmt.query_map([target_path], Self::row_to_target)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn target_path_reference_count(
        &self,
        target_path: &str,
        excluding_id: Option<&str>,
    ) -> Result<u64> {
        let conn = self.open()?;
        let count: i64 = if let Some(id) = excluding_id {
            conn.query_row(
                "SELECT COUNT(*) FROM targets WHERE target_path=?1 AND id<>?2 AND status<>'disabled'",
                params![target_path, id],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM targets WHERE target_path=?1 AND status<>'disabled'",
                [target_path],
                |row| row.get(0),
            )?
        };
        Ok(count.max(0) as u64)
    }

    pub fn add_custom_tool(&self, tool: &CustomTool) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            r#"INSERT INTO custom_tools(key,label,global_dir,project_dir,detect_dir,sync_mode,enabled)
               VALUES (?1,?2,?3,?4,?5,?6,?7)
               ON CONFLICT(key) DO UPDATE SET label=excluded.label,global_dir=excluded.global_dir,
               project_dir=excluded.project_dir,detect_dir=excluded.detect_dir,sync_mode=excluded.sync_mode,
               enabled=excluded.enabled"#,
            params![
                tool.key,
                tool.label,
                tool.global_dir,
                tool.project_dir,
                tool.detect_dir,
                tool.sync_mode.to_string(),
                tool.enabled as i64,
            ],
        )?;
        Ok(())
    }

    pub fn list_custom_tools(&self) -> Result<Vec<CustomTool>> {
        let conn = self.open()?;
        let mut stmt = conn.prepare(
            "SELECT key,label,global_dir,project_dir,detect_dir,sync_mode,enabled FROM custom_tools ORDER BY key",
        )?;
        let rows = stmt.query_map([], |row| {
            let mode: String = row.get(5)?;
            Ok(CustomTool {
                key: row.get(0)?,
                label: row.get(1)?,
                global_dir: row.get(2)?,
                project_dir: row.get(3)?,
                detect_dir: row.get(4)?,
                sync_mode: mode.parse().unwrap_or(SyncMode::Auto),
                enabled: row.get::<_, i64>(6)? != 0,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_custom_tool(&self, key: &str) -> Result<Option<CustomTool>> {
        Ok(self.list_custom_tools()?.into_iter().find(|t| t.key == key))
    }

    pub fn remove_custom_tool(&self, key: &str) -> Result<()> {
        let conn = self.open()?;
        conn.execute("DELETE FROM custom_tools WHERE key=?1", [key])?;
        Ok(())
    }

    pub fn add_recycle_snapshot(&self, snapshot: &RecycleSnapshot) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let raw = serde_json::to_string(snapshot)?;
        let conn = self.open()?;
        conn.execute(
            "INSERT INTO recycle(id,snapshot_json,deleted_at) VALUES (?1,?2,?3)",
            params![id, raw, snapshot.deleted_at],
        )?;
        Ok(id)
    }

    pub fn list_recycle(&self) -> Result<Vec<(String, RecycleSnapshot)>> {
        let conn = self.open()?;
        let mut stmt =
            conn.prepare("SELECT id,snapshot_json FROM recycle ORDER BY deleted_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let raw: String = row.get(1)?;
            Ok((id, raw))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, raw) = row?;
            let snapshot = serde_json::from_str::<RecycleSnapshot>(&raw)
                .with_context(|| format!("parse recycle snapshot {id}"))?;
            out.push((id, snapshot));
        }
        Ok(out)
    }

    pub fn get_recycle(&self, id: &str) -> Result<Option<RecycleSnapshot>> {
        let conn = self.open()?;
        let raw: Option<String> = conn
            .query_row(
                "SELECT snapshot_json FROM recycle WHERE id=?1",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        raw.map(|v| serde_json::from_str(&v).map_err(Into::into))
            .transpose()
    }

    pub fn delete_recycle(&self, id: &str) -> Result<()> {
        let conn = self.open()?;
        conn.execute("DELETE FROM recycle WHERE id=?1", [id])?;
        Ok(())
    }
}
