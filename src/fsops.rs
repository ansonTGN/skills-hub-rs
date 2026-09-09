use std::collections::VecDeque;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::models::{SyncMode, SyncOutcome};

const SKIP_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    ".venv",
    "venv",
    "__pycache__",
    ".idea",
    ".vscode",
];

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub fn sanitize_skill_name(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_dash = false;
    for c in input.trim().chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            out.push(c);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

pub fn skill_metadata(skill_dir: &Path) -> Result<(String, Option<String>)> {
    let skill_file = skill_dir.join("SKILL.md");
    if !skill_file.is_file() {
        bail!("SKILL.md not found in {}", skill_dir.display());
    }
    let raw = fs::read_to_string(&skill_file)
        .with_context(|| format!("read {}", skill_file.display()))?;

    let mut name = None;
    let mut description = None;
    if raw.starts_with("---") {
        let mut lines = raw.lines();
        let _ = lines.next();
        for line in lines {
            if line.trim() == "---" {
                break;
            }
            if let Some(value) = line.strip_prefix("name:") {
                name = Some(unquote(value.trim()).to_string());
            } else if let Some(value) = line.strip_prefix("description:") {
                description = Some(unquote(value.trim()).to_string()).filter(|s| !s.is_empty());
            }
        }
    }

    let fallback = skill_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("skill");
    let resolved = sanitize_skill_name(name.as_deref().unwrap_or(fallback));
    if resolved.is_empty() {
        bail!(
            "could not derive a valid skill name from {}",
            skill_dir.display()
        );
    }
    Ok((resolved, description))
}

fn unquote(value: &str) -> &str {
    if value.len() >= 2 {
        let b = value.as_bytes();
        if (b[0] == b'"' && b[value.len() - 1] == b'"')
            || (b[0] == b'\'' && b[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }
    value
}

pub fn discover_skills(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    if root.join("SKILL.md").is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    if !root.is_dir() {
        bail!("not a directory: {}", root.display());
    }

    let mut found = Vec::new();
    let mut queue = VecDeque::from([(root.to_path_buf(), 0usize)]);
    while let Some((dir, depth)) = queue.pop_front() {
        if depth > max_depth {
            continue;
        }
        let read = match fs::read_dir(&dir) {
            Ok(v) => v,
            Err(_) => continue,
        };
        for entry in read.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }
            let basename = entry.file_name();
            let basename = basename.to_string_lossy();
            if SKIP_DIRS.iter().any(|skip| *skip == basename) {
                continue;
            }
            if path.join("SKILL.md").is_file() {
                found.push(path);
            } else if depth < max_depth {
                queue.push_back((path, depth + 1));
            }
        }
    }
    found.sort();
    Ok(found)
}

pub fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    if !source.is_dir() {
        bail!("copy source is not a directory: {}", source.display());
    }
    fs::create_dir_all(target).with_context(|| format!("create {}", target.display()))?;
    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(source)?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        if rel.components().next().and_then(|c| c.as_os_str().to_str()) == Some(".git") {
            continue;
        }
        let dest = target.join(rel);
        let ft = entry.file_type();
        if ft.is_dir() {
            fs::create_dir_all(&dest)?;
        } else if ft.is_symlink() {
            let link_target = fs::read_link(path)?;
            create_symlink_generic(&link_target, &dest, path.is_dir())?;
        } else if ft.is_file() {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &dest)
                .with_context(|| format!("copy {} -> {}", path.display(), dest.display()))?;
            let _ = fs::set_permissions(&dest, fs::metadata(path)?.permissions());
        }
    }
    Ok(())
}

fn lexical_absolute(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut out = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    Ok(out)
}

pub fn canonicalish(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).with_context(|| format!("canonicalize {}", path.display()));
    }
    if let Some(parent) = path.parent() {
        if parent.exists() {
            let base = fs::canonicalize(parent)?;
            if let Some(name) = path.file_name() {
                return Ok(base.join(name));
            }
        }
    }
    lexical_absolute(path)
}

pub fn paths_overlap(a: &Path, b: &Path) -> Result<bool> {
    let a = canonicalish(a)?;
    let b = canonicalish(b)?;
    Ok(a == b || a.starts_with(&b) || b.starts_with(&a))
}

pub fn remove_path_any(path: &Path) -> Result<()> {
    let md = match fs::symlink_metadata(path) {
        Ok(v) => v,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    let ft = md.file_type();
    if ft.is_symlink() {
        if fs::remove_file(path).is_err() {
            fs::remove_dir(path)?;
        }
        return Ok(());
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if md.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            fs::remove_dir(path)
                .with_context(|| format!("remove reparse point {}", path.display()))?;
            return Ok(());
        }
    }

    if ft.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("remove dir {}", path.display()))?;
    } else {
        fs::remove_file(path).with_context(|| format!("remove file {}", path.display()))?;
    }
    Ok(())
}

#[cfg(unix)]
fn create_symlink_dir(source: &Path, target: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}

#[cfg(windows)]
fn create_symlink_dir(source: &Path, target: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(source, target)?;
    Ok(())
}

#[cfg(unix)]
fn create_symlink_generic(source: &Path, target: &Path, _source_is_dir: bool) -> Result<()> {
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}

#[cfg(windows)]
fn create_symlink_generic(source: &Path, target: &Path, source_is_dir: bool) -> Result<()> {
    if source_is_dir {
        std::os::windows::fs::symlink_dir(source, target)?;
    } else {
        std::os::windows::fs::symlink_file(source, target)?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn create_junction(_source: &Path, _target: &Path) -> Result<()> {
    bail!("junction mode is available only on Windows")
}

#[cfg(windows)]
fn create_junction(source: &Path, target: &Path) -> Result<()> {
    let status = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(target)
        .arg(source)
        .status()
        .context("execute mklink /J")?;
    if !status.success() {
        bail!("mklink /J failed with status {status}");
    }
    Ok(())
}

pub fn sync_dir(
    source: &Path,
    target: &Path,
    requested: SyncMode,
    force_copy: bool,
    overwrite: bool,
) -> Result<SyncOutcome> {
    if !source.is_dir() {
        bail!("managed source does not exist: {}", source.display());
    }
    if paths_overlap(source, target)? {
        bail!(
            "refusing sync because source and target overlap: {} <-> {}",
            source.display(),
            target.display()
        );
    }
    if fs::symlink_metadata(target).is_ok() {
        if !overwrite {
            bail!("TARGET_EXISTS|{}", target.display());
        }
        remove_path_any(target)?;
    }
    let parent = target.parent().context("target has no parent")?;
    fs::create_dir_all(parent)?;

    let effective = if force_copy {
        SyncMode::Copy
    } else {
        requested
    };
    match effective {
        SyncMode::Copy => {
            copy_dir(source, target)?;
            Ok(SyncOutcome {
                mode_used: SyncMode::Copy,
                target_path: target.to_path_buf(),
            })
        }
        SyncMode::Symlink => {
            create_symlink_dir(source, target)?;
            Ok(SyncOutcome {
                mode_used: SyncMode::Symlink,
                target_path: target.to_path_buf(),
            })
        }
        SyncMode::Junction => {
            create_junction(source, target)?;
            Ok(SyncOutcome {
                mode_used: SyncMode::Junction,
                target_path: target.to_path_buf(),
            })
        }
        SyncMode::Auto => {
            #[cfg(windows)]
            {
                if create_junction(source, target).is_ok() {
                    return Ok(SyncOutcome {
                        mode_used: SyncMode::Junction,
                        target_path: target.to_path_buf(),
                    });
                }
                let _ = remove_path_any(target);
                if create_symlink_dir(source, target).is_ok() {
                    return Ok(SyncOutcome {
                        mode_used: SyncMode::Symlink,
                        target_path: target.to_path_buf(),
                    });
                }
                let _ = remove_path_any(target);
            }
            #[cfg(not(windows))]
            {
                if create_symlink_dir(source, target).is_ok() {
                    return Ok(SyncOutcome {
                        mode_used: SyncMode::Symlink,
                        target_path: target.to_path_buf(),
                    });
                }
                let _ = remove_path_any(target);
            }
            copy_dir(source, target)?;
            Ok(SyncOutcome {
                mode_used: SyncMode::Copy,
                target_path: target.to_path_buf(),
            })
        }
    }
}

pub fn dir_hash(root: &Path) -> Result<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(root)?.to_path_buf();
        if rel.components().next().and_then(|c| c.as_os_str().to_str()) == Some(".git") {
            continue;
        }
        files.push((rel, entry.path().to_path_buf()));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (rel, path) in files {
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update([0]);
        let mut file = fs::File::open(&path)?;
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        hasher.update([0xff]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn atomic_replace_dir(source: &Path, target: &Path) -> Result<()> {
    let parent = target.parent().context("target has no parent")?;
    fs::create_dir_all(parent)?;
    let staging = parent.join(format!(".staging-{}", uuid::Uuid::new_v4()));
    let backup = parent.join(format!(".backup-{}", uuid::Uuid::new_v4()));
    copy_dir(source, &staging)?;

    let had_target = target.exists();
    if had_target {
        fs::rename(target, &backup).with_context(|| format!("backup {}", target.display()))?;
    }
    if let Err(err) = fs::rename(&staging, target) {
        if had_target {
            let _ = fs::rename(&backup, target);
        }
        let _ = remove_path_any(&staging);
        return Err(err).with_context(|| format!("activate {}", target.display()));
    }
    if had_target {
        let _ = remove_path_any(&backup);
    }
    Ok(())
}

pub fn run_git_clone(url: &str, revision: Option<&str>, destination: &Path) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("clone").arg("--depth").arg("1");
    if let Some(rev) = revision.filter(|s| !s.trim().is_empty()) {
        cmd.args(["--branch", rev]);
    }
    cmd.arg("--").arg(url).arg(destination);
    let status = cmd.status().context("run git clone")?;
    if !status.success() {
        bail!("git clone failed with status {status}");
    }
    Ok(())
}
