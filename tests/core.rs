use std::fs;

use skills_hub_rs::models::{CustomTool, InstallOptions, Scope, SyncMode};
use skills_hub_rs::SkillsHub;
use tempfile::TempDir;

fn make_skill(root: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let dir = root.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: test skill\n---\n\n{body}\n"),
    )
    .unwrap();
    fs::write(dir.join("notes.txt"), body).unwrap();
    dir
}

#[test]
fn local_install_sync_disable_enable_and_recycle_restore() {
    let temp = TempDir::new().unwrap();
    let hub = SkillsHub::open(Some(temp.path().join("hub"))).unwrap();
    let source_root = temp.path().join("sources");
    fs::create_dir_all(&source_root).unwrap();
    let skill_dir = make_skill(&source_root, "demo-skill", "hello");

    let target_root = temp.path().join("tool-skills");
    hub.add_custom_tool(CustomTool {
        key: "test_tool".into(),
        label: "Test Tool".into(),
        global_dir: target_root.to_string_lossy().to_string(),
        project_dir: Some(".test/skills".into()),
        detect_dir: None,
        sync_mode: SyncMode::Copy,
        enabled: true,
    })
    .unwrap();

    let installed = hub
        .install_local(
            &skill_dir,
            &InstallOptions {
                tags: vec!["demo".into(), "rust".into()],
                tools: vec!["test_tool".into()],
                scope: Scope::Global,
                project: None,
                mode: SyncMode::Auto,
                overwrite: false,
            },
        )
        .unwrap();
    assert_eq!(installed.len(), 1);
    assert!(hub.skills_dir.join("demo-skill/SKILL.md").is_file());
    assert!(target_root.join("demo-skill/SKILL.md").is_file());

    hub.disable_skill("demo-skill").unwrap();
    assert!(!target_root.join("demo-skill").exists());
    assert!(hub.skills_dir.join("demo-skill/SKILL.md").is_file());

    hub.enable_skill("demo-skill").unwrap();
    assert!(target_root.join("demo-skill/SKILL.md").is_file());

    let recycle_id = hub.remove_skill("demo-skill", false).unwrap().unwrap();
    assert!(hub.store.get_skill("demo-skill").unwrap().is_none());
    assert!(!hub.skills_dir.join("demo-skill").exists());

    hub.restore_recycle(&recycle_id).unwrap();
    assert!(hub.store.get_skill("demo-skill").unwrap().is_some());
    assert!(hub.skills_dir.join("demo-skill/SKILL.md").is_file());
    assert!(target_root.join("demo-skill/SKILL.md").is_file());
}

#[test]
fn local_update_refreshes_copy_target() {
    let temp = TempDir::new().unwrap();
    let hub = SkillsHub::open(Some(temp.path().join("hub"))).unwrap();
    let source_root = temp.path().join("sources");
    fs::create_dir_all(&source_root).unwrap();
    let skill_dir = make_skill(&source_root, "update-me", "v1");
    let target_root = temp.path().join("tool-skills");

    hub.add_custom_tool(CustomTool {
        key: "copy_tool".into(),
        label: "Copy Tool".into(),
        global_dir: target_root.to_string_lossy().to_string(),
        project_dir: None,
        detect_dir: None,
        sync_mode: SyncMode::Copy,
        enabled: true,
    })
    .unwrap();

    hub.install_local(
        &skill_dir,
        &InstallOptions {
            tools: vec!["copy_tool".into()],
            ..InstallOptions::default()
        },
    )
    .unwrap();

    fs::write(skill_dir.join("notes.txt"), "v2").unwrap();
    assert_eq!(hub.update_skill("update-me").unwrap(), "updated");
    assert_eq!(
        fs::read_to_string(target_root.join("update-me/notes.txt")).unwrap(),
        "v2"
    );
    assert_eq!(hub.update_skill("update-me").unwrap(), "up-to-date");
}

#[test]
fn project_scope_uses_project_relative_directory() {
    let temp = TempDir::new().unwrap();
    let hub = SkillsHub::open(Some(temp.path().join("hub"))).unwrap();
    let source_root = temp.path().join("sources");
    fs::create_dir_all(&source_root).unwrap();
    let skill_dir = make_skill(&source_root, "project-skill", "hello");
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();

    hub.add_custom_tool(CustomTool {
        key: "project_tool".into(),
        label: "Project Tool".into(),
        global_dir: temp
            .path()
            .join("global-unused")
            .to_string_lossy()
            .to_string(),
        project_dir: Some(".agent/skills".into()),
        detect_dir: None,
        sync_mode: SyncMode::Copy,
        enabled: true,
    })
    .unwrap();

    hub.install_local(
        &skill_dir,
        &InstallOptions {
            tools: vec!["project_tool".into()],
            scope: Scope::Project,
            project: Some(project.clone()),
            mode: SyncMode::Copy,
            ..InstallOptions::default()
        },
    )
    .unwrap();
    assert!(project
        .join(".agent/skills/project-skill/SKILL.md")
        .is_file());
}

#[test]
fn builtin_catalog_has_expected_tools_and_cursor_forces_copy() {
    let tools = skills_hub_rs::tools::builtin_tools();
    assert_eq!(tools.len(), 47);

    let cursor = tools.iter().find(|tool| tool.key == "cursor").unwrap();
    assert!(cursor.force_copy);
    assert_eq!(cursor.global_dir, ".cursor/skills");

    let codex = tools.iter().find(|tool| tool.key == "codex").unwrap();
    assert!(!codex.force_copy);
    assert_eq!(codex.project_dir, Some(".agents/skills"));
}

#[test]
fn shared_physical_target_is_removed_only_after_last_relation() {
    let temp = TempDir::new().unwrap();
    let hub = SkillsHub::open(Some(temp.path().join("hub"))).unwrap();
    let source_root = temp.path().join("sources");
    fs::create_dir_all(&source_root).unwrap();
    let skill_dir = make_skill(&source_root, "shared-skill", "hello");
    let shared_root = temp.path().join("shared-skills");

    for (key, label) in [("tool_a", "Tool A"), ("tool_b", "Tool B")] {
        hub.add_custom_tool(CustomTool {
            key: key.into(),
            label: label.into(),
            global_dir: shared_root.to_string_lossy().to_string(),
            project_dir: None,
            detect_dir: None,
            sync_mode: SyncMode::Copy,
            enabled: true,
        })
        .unwrap();
    }

    hub.install_local(&skill_dir, &InstallOptions::default())
        .unwrap();
    hub.sync_skill(
        "shared-skill",
        "tool_a",
        Scope::Global,
        None,
        SyncMode::Copy,
        false,
    )
    .unwrap();
    hub.sync_skill(
        "shared-skill",
        "tool_b",
        Scope::Global,
        None,
        SyncMode::Copy,
        false,
    )
    .unwrap();

    let physical = shared_root.join("shared-skill");
    assert!(physical.join("SKILL.md").is_file());

    hub.unsync_skill("shared-skill", "tool_a", Scope::Global, None)
        .unwrap();
    assert!(physical.join("SKILL.md").is_file());

    hub.unsync_skill("shared-skill", "tool_b", Scope::Global, None)
        .unwrap();
    assert!(!physical.exists());
}

#[test]
fn nested_discovery_skips_build_and_dependency_directories() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let expected = make_skill(&root.join("skills/category"), "nested-skill", "ok");
    let ignored = make_skill(&root.join("node_modules/pkg"), "ignored-skill", "ignore");
    assert!(ignored.join("SKILL.md").is_file());

    let found = skills_hub_rs::fsops::discover_skills(root, 4).unwrap();
    assert_eq!(found, vec![expected]);
}
