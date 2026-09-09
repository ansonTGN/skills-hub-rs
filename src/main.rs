use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use serde_json::json;
use skills_hub_rs::models::{CustomTool, InstallOptions, Scope, SyncMode};
use skills_hub_rs::{online, SkillsHub};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "skills-hub-rs")]
#[command(
    version,
    about = "Install Agent Skills once and sync them to multiple AI coding tools"
)]
struct Cli {
    /// Override the managed data directory (default: ~/.skillshub-rs)
    #[arg(long, global = true, env = "SKILLS_HUB_HOME")]
    home: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize storage and print its paths
    Init,
    /// List managed skills
    List {
        #[arg(long)]
        json: bool,
    },
    /// Show one skill, its tags, and sync targets
    Show {
        skill: String,
        #[arg(long)]
        json: bool,
    },
    /// List files contained in a managed skill
    Files { skill: String },
    /// List built-in and custom tool adapters
    Tools {
        /// Only show tools detected on this computer
        #[arg(long)]
        detected: bool,
        #[arg(long)]
        json: bool,
    },
    /// Discover SKILL.md directories in installed tool locations
    Scan {
        #[arg(long)]
        tool: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Install one or more Skills from a local directory
    #[command(alias = "import")]
    AddLocal {
        path: PathBuf,
        #[command(flatten)]
        install: InstallArgs,
    },
    /// Clone a Git repository and install all detected Skills
    AddGit {
        url: String,
        /// Branch or tag passed to git clone --branch
        #[arg(long = "ref")]
        revision: Option<String>,
        /// Search only below this repository-relative directory
        #[arg(long)]
        subdir: Option<PathBuf>,
        #[command(flatten)]
        install: InstallArgs,
    },
    /// Sync a managed Skill to one tool
    Sync {
        skill: String,
        #[arg(long)]
        tool: String,
        #[arg(long, default_value = "global")]
        scope: String,
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long, default_value = "auto")]
        mode: String,
        #[arg(long)]
        overwrite: bool,
    },
    /// Remove one Skill-to-tool synchronization relation
    Unsync {
        skill: String,
        #[arg(long)]
        tool: String,
        #[arg(long, default_value = "global")]
        scope: String,
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Disable a Skill and remove its live tool targets without deleting the managed copy
    Disable { skill: String },
    /// Re-enable a Skill and recreate its configured targets
    Enable { skill: String },
    /// Update one Skill from its source, or all updateable Skills
    Update { skill: Option<String> },
    /// Delete a Skill. By default it goes to the local recoverable recycle bin.
    Remove {
        skill: String,
        #[arg(long)]
        permanent: bool,
    },
    /// Manage the recoverable local recycle bin
    Recycle {
        #[command(subcommand)]
        command: RecycleCommand,
    },
    /// Search the public skills.sh index
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Add or remove tags on a managed Skill
    Tag {
        #[command(subcommand)]
        command: TagCommand,
    },
    /// Manage custom AI tool skill directories
    CustomTool {
        #[command(subcommand)]
        command: CustomToolCommand,
    },
}

#[derive(Args, Debug)]
struct InstallArgs {
    /// Add a tag; repeat for multiple tags
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// Sync immediately to this tool; repeat for multiple tools
    #[arg(long = "tool")]
    tools: Vec<String>,
    #[arg(long, default_value = "global")]
    scope: String,
    #[arg(long)]
    project: Option<PathBuf>,
    #[arg(long, default_value = "auto")]
    mode: String,
    /// Allow replacing an existing filesystem target during initial synchronization
    #[arg(long)]
    overwrite: bool,
}

impl InstallArgs {
    fn into_options(self) -> Result<InstallOptions> {
        Ok(InstallOptions {
            tags: self.tags,
            tools: self.tools,
            scope: self.scope.parse()?,
            project: self.project,
            mode: self.mode.parse()?,
            overwrite: self.overwrite,
        })
    }
}

#[derive(Subcommand, Debug)]
enum RecycleCommand {
    List,
    Restore {
        id: String,
    },
    /// Purge entries older than 30 days; --all purges every entry
    Purge {
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand, Debug)]
enum TagCommand {
    Add { skill: String, tag: String },
    Remove { skill: String, tag: String },
}

#[derive(Subcommand, Debug)]
enum CustomToolCommand {
    List,
    Add {
        key: String,
        #[arg(long)]
        label: String,
        /// Global skills directory; supports ~/...
        #[arg(long)]
        global_dir: String,
        /// Project-relative skills directory
        #[arg(long)]
        project_dir: Option<String>,
        /// Directory used for local detection; supports ~/...
        #[arg(long)]
        detect_dir: Option<String>,
        #[arg(long, default_value = "auto")]
        mode: String,
        #[arg(long, default_value_t = true)]
        enabled: bool,
    },
    Remove {
        key: String,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let hub = SkillsHub::open(cli.home)?;

    match cli.command {
        Command::Init => {
            println!("Skills Hub Rust initialized");
            println!("  root:    {}", hub.root.display());
            println!("  skills:  {}", hub.skills_dir.display());
            println!("  db:      {}", hub.store.db_path().display());
            println!("  recycle: {}", hub.recycle_dir.display());
        }
        Command::List { json: as_json } => {
            let skills = hub.store.list_skills()?;
            if as_json {
                println!("{}", serde_json::to_string_pretty(&skills)?);
            } else if skills.is_empty() {
                println!("No managed skills.");
            } else {
                for skill in skills {
                    let tags = hub.store.tags_for_skill(&skill.id)?;
                    let targets = hub.store.list_targets(&skill.id)?;
                    println!(
                        "{:<28} {:<8} source={:<5} tags=[{}] targets={}",
                        skill.name,
                        if skill.enabled { "enabled" } else { "disabled" },
                        skill.source_type,
                        tags.join(","),
                        targets.len()
                    );
                }
            }
        }
        Command::Show {
            skill,
            json: as_json,
        } => {
            let skill = hub.store.get_skill(&skill)?.context("skill not found")?;
            let tags = hub.store.tags_for_skill(&skill.id)?;
            let targets = hub.store.list_targets(&skill.id)?;
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "skill": skill,
                        "tags": tags,
                        "targets": targets,
                    }))?
                );
            } else {
                println!("name:        {}", skill.name);
                println!("id:          {}", skill.id);
                println!("enabled:     {}", skill.enabled);
                println!("source:      {}", skill.source_type);
                println!(
                    "source_ref:  {}",
                    skill.source_ref.as_deref().unwrap_or("-")
                );
                println!(
                    "subdir:      {}",
                    skill.source_subdir.as_deref().unwrap_or("-")
                );
                println!(
                    "revision:    {}",
                    skill.source_revision.as_deref().unwrap_or("-")
                );
                println!("central:     {}", skill.central_path);
                println!(
                    "description: {}",
                    skill.description.as_deref().unwrap_or("-")
                );
                println!(
                    "tags:        {}",
                    if tags.is_empty() {
                        "-".into()
                    } else {
                        tags.join(", ")
                    }
                );
                println!("targets:");
                if targets.is_empty() {
                    println!("  - none");
                } else {
                    for t in targets {
                        println!(
                            "  - {} scope={} mode={} status={} path={}",
                            t.tool, t.scope, t.mode, t.status, t.target_path
                        );
                        if let Some(err) = t.last_error {
                            println!("    error={err}");
                        }
                    }
                }
            }
        }
        Command::Files { skill } => {
            let skill = hub.store.get_skill(&skill)?.context("skill not found")?;
            let root = PathBuf::from(&skill.central_path);
            for entry in WalkDir::new(&root).follow_links(false) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    println!("{}", entry.path().strip_prefix(&root)?.display());
                }
            }
        }
        Command::Tools {
            detected,
            json: as_json,
        } => {
            let tools = hub.list_tools()?;
            let values = tools
                .into_iter()
                .filter(|tool| {
                    !detected
                        || tool
                            .detect_dir
                            .as_deref()
                            .map(|p| p.exists())
                            .unwrap_or(false)
                })
                .map(|tool| {
                    json!({
                        "key": tool.key,
                        "label": tool.label,
                        "builtin": tool.builtin,
                        "detected": tool.detect_dir.as_deref().map(|p| p.exists()).unwrap_or(false),
                        "global_dir": tool.global_dir,
                        "project_dir": tool.project_dir,
                        "force_copy": tool.force_copy,
                        "sync_mode": tool.sync_mode.to_string(),
                    })
                })
                .collect::<Vec<_>>();
            if as_json {
                println!("{}", serde_json::to_string_pretty(&values)?);
            } else {
                for v in values {
                    println!(
                        "{:<18} {:<20} detected={:<5} global={}",
                        v["key"].as_str().unwrap_or(""),
                        v["label"].as_str().unwrap_or(""),
                        v["detected"].as_bool().unwrap_or(false),
                        v["global_dir"].as_str().unwrap_or(""),
                    );
                }
            }
        }
        Command::Scan {
            tool,
            json: as_json,
        } => {
            let found = hub.scan_tool(tool.as_deref())?;
            if as_json {
                println!("{}", serde_json::to_string_pretty(&found)?);
            } else if found.is_empty() {
                println!("No importable SKILL.md directories found.");
            } else {
                for item in found {
                    println!(
                        "{:<18} {:<28} {}{}",
                        item.tool.as_deref().unwrap_or("-"),
                        item.name,
                        item.path,
                        if item.is_link { " [link]" } else { "" }
                    );
                }
            }
        }
        Command::AddLocal { path, install } => {
            let records = hub.install_local(&path, &install.into_options()?)?;
            for skill in records {
                println!("installed {} -> {}", skill.name, skill.central_path);
            }
        }
        Command::AddGit {
            url,
            revision,
            subdir,
            install,
        } => {
            let records = hub.install_git(
                &url,
                revision.as_deref(),
                subdir.as_deref(),
                &install.into_options()?,
            )?;
            for skill in records {
                println!("installed {} -> {}", skill.name, skill.central_path);
            }
        }
        Command::Sync {
            skill,
            tool,
            scope,
            project,
            mode,
            overwrite,
        } => {
            let record = hub.sync_skill(
                &skill,
                &tool,
                scope.parse::<Scope>()?,
                project.as_deref(),
                mode.parse::<SyncMode>()?,
                overwrite,
            )?;
            println!(
                "synced {} -> {} ({})",
                skill, record.target_path, record.mode
            );
        }
        Command::Unsync {
            skill,
            tool,
            scope,
            project,
        } => {
            hub.unsync_skill(&skill, &tool, scope.parse()?, project.as_deref())?;
            println!("unsynced {skill} from {tool}");
        }
        Command::Disable { skill } => {
            hub.disable_skill(&skill)?;
            println!("disabled {skill}");
        }
        Command::Enable { skill } => {
            hub.enable_skill(&skill)?;
            println!("enabled {skill}");
        }
        Command::Update { skill } => {
            if let Some(skill) = skill {
                println!("{}: {}", skill, hub.update_skill(&skill)?);
            } else {
                for (skill, status) in hub.update_all()? {
                    println!("{skill}: {status}");
                }
            }
        }
        Command::Remove { skill, permanent } => match hub.remove_skill(&skill, permanent)? {
            Some(id) => println!("moved {skill} to recycle bin: {id}"),
            None => println!("permanently removed {skill}"),
        },
        Command::Recycle { command } => match command {
            RecycleCommand::List => {
                let entries = hub.list_recycle()?;
                if entries.is_empty() {
                    println!("Recycle bin is empty.");
                }
                for (id, snapshot) in entries {
                    println!(
                        "{}  {:<28} deleted_at={}",
                        id, snapshot.skill.name, snapshot.deleted_at
                    );
                }
            }
            RecycleCommand::Restore { id } => {
                let skill = hub.restore_recycle(&id)?;
                println!("restored {}", skill.name);
            }
            RecycleCommand::Purge { all } => {
                println!("purged {} recycle entries", hub.purge_recycle(all)?);
            }
        },
        Command::Search {
            query,
            limit,
            json: as_json,
        } => {
            let results = online::search(&query, limit)?;
            if as_json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for item in results {
                    println!(
                        "{:<36} installs={:<8} {}",
                        item.name, item.installs, item.source_url
                    );
                }
            }
        }
        Command::Tag { command } => match command {
            TagCommand::Add { skill, tag } => {
                let record = hub.store.get_skill(&skill)?.context("skill not found")?;
                hub.store.add_tag(&record.id, &tag)?;
                println!("tagged {} with {}", record.name, tag);
            }
            TagCommand::Remove { skill, tag } => {
                let record = hub.store.get_skill(&skill)?.context("skill not found")?;
                hub.store.remove_tag(&record.id, &tag)?;
                println!("removed tag {} from {}", tag, record.name);
            }
        },
        Command::CustomTool { command } => match command {
            CustomToolCommand::List => {
                for tool in hub.store.list_custom_tools()? {
                    println!(
                        "{:<18} {:<22} enabled={} mode={} global={}",
                        tool.key, tool.label, tool.enabled, tool.sync_mode, tool.global_dir
                    );
                }
            }
            CustomToolCommand::Add {
                key,
                label,
                global_dir,
                project_dir,
                detect_dir,
                mode,
                enabled,
            } => {
                hub.add_custom_tool(CustomTool {
                    key: key.clone(),
                    label,
                    global_dir,
                    project_dir,
                    detect_dir,
                    sync_mode: mode.parse()?,
                    enabled,
                })?;
                println!("saved custom tool {key}");
            }
            CustomToolCommand::Remove { key } => {
                hub.remove_custom_tool(&key)?;
                println!("removed custom tool {key}");
            }
        },
    }
    Ok(())
}
