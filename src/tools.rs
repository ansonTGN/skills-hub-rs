use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::models::{CustomTool, ResolvedTool, SyncMode, ToolAdapter};

pub fn builtin_tools() -> Vec<ToolAdapter> {
    vec![
        ToolAdapter {
            key: "cursor",
            label: "Cursor",
            global_dir: ".cursor/skills",
            project_dir: Some(".agents/skills"),
            detect_dir: ".cursor",
            force_copy: true,
        },
        ToolAdapter {
            key: "claude_code",
            label: "Claude Code",
            global_dir: ".claude/skills",
            project_dir: Some(".claude/skills"),
            detect_dir: ".claude",
            force_copy: false,
        },
        ToolAdapter {
            key: "codex",
            label: "Codex",
            global_dir: ".codex/skills",
            project_dir: Some(".agents/skills"),
            detect_dir: ".codex",
            force_copy: false,
        },
        ToolAdapter {
            key: "deepseek_harness",
            label: "DeepSeek Harness",
            global_dir: ".dsh/skills",
            project_dir: Some(".dsh/skills"),
            detect_dir: ".dsh",
            force_copy: false,
        },
        ToolAdapter {
            key: "opencode",
            label: "OpenCode",
            global_dir: ".config/opencode/skills",
            project_dir: Some(".agents/skills"),
            detect_dir: ".config/opencode",
            force_copy: false,
        },
        ToolAdapter {
            key: "antigravity",
            label: "Antigravity",
            global_dir: ".gemini/config/skills",
            project_dir: Some(".agents/skills"),
            detect_dir: ".gemini/config",
            force_copy: false,
        },
        ToolAdapter {
            key: "amp",
            label: "Amp",
            global_dir: ".config/agents/skills",
            project_dir: Some(".agents/skills"),
            detect_dir: ".config/agents",
            force_copy: false,
        },
        ToolAdapter {
            key: "kimi_cli",
            label: "Kimi Code CLI",
            global_dir: ".kimi-code/skills",
            project_dir: Some(".kimi-code/skills"),
            detect_dir: ".kimi-code",
            force_copy: false,
        },
        ToolAdapter {
            key: "augment",
            label: "Augment",
            global_dir: ".augment/skills",
            project_dir: Some(".augment/skills"),
            detect_dir: ".augment",
            force_copy: false,
        },
        ToolAdapter {
            key: "openclaw",
            label: "OpenClaw",
            global_dir: ".openclaw/skills",
            project_dir: Some("skills"),
            detect_dir: ".openclaw",
            force_copy: false,
        },
        ToolAdapter {
            key: "copaw",
            label: "Copaw",
            global_dir: ".copaw/skill_pool",
            project_dir: Some(".copaw/skill_pool"),
            detect_dir: ".copaw",
            force_copy: false,
        },
        ToolAdapter {
            key: "cline",
            label: "Cline",
            global_dir: ".agents/skills",
            project_dir: Some(".agents/skills"),
            detect_dir: ".agents",
            force_copy: false,
        },
        ToolAdapter {
            key: "codebuddy",
            label: "CodeBuddy",
            global_dir: ".codebuddy/skills",
            project_dir: Some(".codebuddy/skills"),
            detect_dir: ".codebuddy",
            force_copy: false,
        },
        ToolAdapter {
            key: "codewhale",
            label: "CodeWhale",
            global_dir: ".codewhale/skills",
            project_dir: Some(".codewhale/skills"),
            detect_dir: ".codewhale",
            force_copy: false,
        },
        ToolAdapter {
            key: "workbuddy",
            label: "WorkBuddy",
            global_dir: ".workbuddy/skills",
            project_dir: None,
            detect_dir: ".workbuddy",
            force_copy: false,
        },
        ToolAdapter {
            key: "command_code",
            label: "Command Code",
            global_dir: ".commandcode/skills",
            project_dir: Some(".commandcode/skills"),
            detect_dir: ".commandcode",
            force_copy: false,
        },
        ToolAdapter {
            key: "continue",
            label: "Continue",
            global_dir: ".continue/skills",
            project_dir: Some(".continue/skills"),
            detect_dir: ".continue",
            force_copy: false,
        },
        ToolAdapter {
            key: "crush",
            label: "Crush",
            global_dir: ".config/crush/skills",
            project_dir: Some(".crush/skills"),
            detect_dir: ".config/crush",
            force_copy: false,
        },
        ToolAdapter {
            key: "junie",
            label: "Junie",
            global_dir: ".junie/skills",
            project_dir: Some(".junie/skills"),
            detect_dir: ".junie",
            force_copy: false,
        },
        ToolAdapter {
            key: "iflow_cli",
            label: "iFlow CLI",
            global_dir: ".iflow/skills",
            project_dir: Some(".iflow/skills"),
            detect_dir: ".iflow",
            force_copy: false,
        },
        ToolAdapter {
            key: "kiro_cli",
            label: "Kiro CLI",
            global_dir: ".kiro/skills",
            project_dir: Some(".kiro/skills"),
            detect_dir: ".kiro",
            force_copy: false,
        },
        ToolAdapter {
            key: "kode",
            label: "Kode",
            global_dir: ".kode/skills",
            project_dir: Some(".kode/skills"),
            detect_dir: ".kode",
            force_copy: false,
        },
        ToolAdapter {
            key: "mcpjam",
            label: "MCPJam",
            global_dir: ".mcpjam/skills",
            project_dir: Some(".mcpjam/skills"),
            detect_dir: ".mcpjam",
            force_copy: false,
        },
        ToolAdapter {
            key: "mistral_vibe",
            label: "Mistral Vibe",
            global_dir: ".vibe/skills",
            project_dir: Some(".vibe/skills"),
            detect_dir: ".vibe",
            force_copy: false,
        },
        ToolAdapter {
            key: "mux",
            label: "Mux",
            global_dir: ".mux/skills",
            project_dir: Some(".mux/skills"),
            detect_dir: ".mux",
            force_copy: false,
        },
        ToolAdapter {
            key: "openclaude",
            label: "OpenClaude IDE",
            global_dir: ".openclaude/skills",
            project_dir: Some(".openclaude/skills"),
            detect_dir: ".openclaude",
            force_copy: false,
        },
        ToolAdapter {
            key: "openhands",
            label: "OpenHands",
            global_dir: ".openhands/skills",
            project_dir: Some(".openhands/skills"),
            detect_dir: ".openhands",
            force_copy: false,
        },
        ToolAdapter {
            key: "pi",
            label: "Pi",
            global_dir: ".pi/agent/skills",
            project_dir: Some(".pi/skills"),
            detect_dir: ".pi",
            force_copy: false,
        },
        ToolAdapter {
            key: "qoder",
            label: "Qoder",
            global_dir: ".qoder/skills",
            project_dir: Some(".qoder/skills"),
            detect_dir: ".qoder",
            force_copy: false,
        },
        ToolAdapter {
            key: "qoderwork",
            label: "QoderWork",
            global_dir: ".qoderwork/skills",
            project_dir: Some(".qoderwork/skills"),
            detect_dir: ".qoderwork",
            force_copy: false,
        },
        ToolAdapter {
            key: "qwen_code",
            label: "Qwen Code",
            global_dir: ".qwen/skills",
            project_dir: Some(".qwen/skills"),
            detect_dir: ".qwen",
            force_copy: false,
        },
        ToolAdapter {
            key: "trae",
            label: "Trae",
            global_dir: ".trae/skills",
            project_dir: Some(".trae/skills"),
            detect_dir: ".trae",
            force_copy: false,
        },
        ToolAdapter {
            key: "trae_cn",
            label: "Trae CN",
            global_dir: ".trae-cn/skills",
            project_dir: Some(".trae/skills"),
            detect_dir: ".trae-cn",
            force_copy: false,
        },
        ToolAdapter {
            key: "zencoder",
            label: "Zencoder",
            global_dir: ".zencoder/skills",
            project_dir: Some(".zencoder/skills"),
            detect_dir: ".zencoder",
            force_copy: false,
        },
        ToolAdapter {
            key: "neovate",
            label: "Neovate",
            global_dir: ".neovate/skills",
            project_dir: Some(".neovate/skills"),
            detect_dir: ".neovate",
            force_copy: false,
        },
        ToolAdapter {
            key: "pochi",
            label: "Pochi",
            global_dir: ".pochi/skills",
            project_dir: Some(".pochi/skills"),
            detect_dir: ".pochi",
            force_copy: false,
        },
        ToolAdapter {
            key: "adal",
            label: "AdaL",
            global_dir: ".adal/skills",
            project_dir: Some(".adal/skills"),
            detect_dir: ".adal",
            force_copy: false,
        },
        ToolAdapter {
            key: "kilo_code",
            label: "Kilo Code",
            global_dir: ".kilocode/skills",
            project_dir: Some(".kilocode/skills"),
            detect_dir: ".kilocode",
            force_copy: false,
        },
        ToolAdapter {
            key: "roo_code",
            label: "Roo Code",
            global_dir: ".roo/skills",
            project_dir: Some(".roo/skills"),
            detect_dir: ".roo",
            force_copy: false,
        },
        ToolAdapter {
            key: "goose",
            label: "Goose",
            global_dir: ".config/goose/skills",
            project_dir: Some(".goose/skills"),
            detect_dir: ".config/goose",
            force_copy: false,
        },
        ToolAdapter {
            key: "gemini_cli",
            label: "Gemini CLI",
            global_dir: ".gemini/skills",
            project_dir: Some(".agents/skills"),
            detect_dir: ".gemini",
            force_copy: false,
        },
        ToolAdapter {
            key: "github_copilot",
            label: "GitHub Copilot",
            global_dir: ".copilot/skills",
            project_dir: Some(".agents/skills"),
            detect_dir: ".copilot",
            force_copy: false,
        },
        ToolAdapter {
            key: "clawdbot",
            label: "Clawdbot",
            global_dir: ".clawdbot/skills",
            project_dir: Some(".clawdbot/skills"),
            detect_dir: ".clawdbot",
            force_copy: false,
        },
        ToolAdapter {
            key: "droid",
            label: "Droid",
            global_dir: ".factory/skills",
            project_dir: Some(".factory/skills"),
            detect_dir: ".factory",
            force_copy: false,
        },
        ToolAdapter {
            key: "windsurf",
            label: "Windsurf",
            global_dir: ".codeium/windsurf/skills",
            project_dir: Some(".windsurf/skills"),
            detect_dir: ".codeium/windsurf",
            force_copy: false,
        },
        ToolAdapter {
            key: "moltbot",
            label: "MoltBot",
            global_dir: ".moltbot/skills",
            project_dir: Some(".moltbot/skills"),
            detect_dir: ".moltbot",
            force_copy: false,
        },
        ToolAdapter {
            key: "hermes_agent",
            label: "Hermes Agent",
            global_dir: ".hermes/skills",
            project_dir: None,
            detect_dir: ".hermes",
            force_copy: false,
        },
    ]
}

fn expand_home_relative(home: &Path, relative: &str) -> PathBuf {
    home.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))
}

fn expand_custom_path(home: &Path, raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    if trimmed == "~" {
        return home.to_path_buf();
    }
    if let Some(rest) = trimmed
        .strip_prefix("~/")
        .or_else(|| trimmed.strip_prefix("~\\"))
    {
        return home.join(rest);
    }
    PathBuf::from(trimmed)
}

pub fn resolve_builtin(home: &Path, key: &str) -> Option<ResolvedTool> {
    let adapter = builtin_tools().into_iter().find(|a| a.key == key)?;
    let (global_dir, detect_dir) = if adapter.key == "kimi_cli" {
        if let Ok(kimi_home) = std::env::var("KIMI_CODE_HOME") {
            let root = PathBuf::from(kimi_home);
            (root.join("skills"), Some(root))
        } else {
            (
                expand_home_relative(home, adapter.global_dir),
                Some(expand_home_relative(home, adapter.detect_dir)),
            )
        }
    } else {
        (
            expand_home_relative(home, adapter.global_dir),
            Some(expand_home_relative(home, adapter.detect_dir)),
        )
    };

    Some(ResolvedTool {
        key: adapter.key.to_string(),
        label: adapter.label.to_string(),
        global_dir,
        project_dir: adapter.project_dir.map(PathBuf::from),
        detect_dir,
        sync_mode: SyncMode::Auto,
        force_copy: adapter.force_copy,
        builtin: true,
    })
}

pub fn resolve_custom(home: &Path, custom: &CustomTool) -> ResolvedTool {
    ResolvedTool {
        key: custom.key.clone(),
        label: custom.label.clone(),
        global_dir: expand_custom_path(home, &custom.global_dir),
        project_dir: custom.project_dir.as_deref().map(PathBuf::from),
        detect_dir: custom
            .detect_dir
            .as_deref()
            .map(|p| expand_custom_path(home, p)),
        sync_mode: custom.sync_mode,
        force_copy: custom.sync_mode == SyncMode::Copy,
        builtin: false,
    }
}

pub fn validate_custom_key(key: &str) -> Result<()> {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        bail!("custom tool key is empty")
    };
    if !first.is_ascii_lowercase() {
        bail!("custom tool key must start with a lowercase ASCII letter");
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
        bail!("custom tool key may contain lowercase letters, digits, '_' and '-' only");
    }
    if builtin_tools().iter().any(|t| t.key == key) {
        bail!("custom tool key conflicts with built-in tool {key}");
    }
    Ok(())
}

pub fn resolve_target_root(
    tool: &ResolvedTool,
    scope: crate::models::Scope,
    project: Option<&Path>,
) -> Result<PathBuf> {
    match scope {
        crate::models::Scope::Global => Ok(tool.global_dir.clone()),
        crate::models::Scope::Project => {
            let project = project.context("project scope requires --project PATH")?;
            let relative = tool.project_dir.as_deref().with_context(|| {
                format!("tool {} has no project-level skills directory", tool.key)
            })?;
            Ok(project.join(relative))
        }
    }
}
