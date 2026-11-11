use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ProjectManifest {
    pub name: String,
    pub root_dir: PathBuf,
    pub binaries: Vec<String>,
    pub source_hash: String,
}

#[derive(Deserialize)]
struct CargoToml {
    package: Option<PackageSection>,
    bin: Option<Vec<BinSection>>,
    workspace: Option<WorkspaceSection>,
}

#[derive(Deserialize)]
struct PackageSection {
    name: String,
    default_run: Option<String>,
}

#[derive(Deserialize)]
struct BinSection {
    name: String,
}

#[derive(Deserialize)]
struct WorkspaceSection {
    members: Option<Vec<String>>,
}

pub fn discover_project(start_path: &Path) -> Result<ProjectManifest> {
    let cargo_path = find_cargo_toml(start_path)?;
    let root_dir = cargo_path
        .parent()
        .context("Invalid Cargo.toml parent directory")?
        .to_path_buf();

    let content = fs::read_to_string(&cargo_path)
        .with_context(|| format!("Failed to read {}", cargo_path.display()))?;

    let parsed: CargoToml = toml::from_str(&content)
        .with_context(|| format!("Failed to parse {}", cargo_path.display()))?;

    let (name, binaries) = resolve_name_and_binaries(&parsed, &root_dir)?;
    let source_hash = compute_source_hash(&root_dir)?;

    Ok(ProjectManifest {
        name,
        root_dir,
        binaries,
        source_hash,
    })
}

fn find_cargo_toml(start_path: &Path) -> Result<PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent().unwrap_or(start_path).to_path_buf()
    } else {
        start_path.to_path_buf()
    };

    loop {
        let candidate = current.join("Cargo.toml");
        if candidate.exists() {
            return Ok(candidate);
        }

        if !current.pop() {
            bail!(
                "Could not find Cargo.toml in {} or any parent directory",
                start_path.display()
            );
        }
    }
}

fn resolve_name_and_binaries(
    parsed: &CargoToml,
    root_dir: &Path,
) -> Result<(String, Vec<String>)> {
    if let Some(pkg) = &parsed.package {
        let mut bins = Vec::new();
        if let Some(bin_list) = &parsed.bin {
            for b in bin_list {
                bins.push(b.name.clone());
            }
        }

        if bins.is_empty() {
            if let Some(def_run) = &pkg.default_run {
                bins.push(def_run.clone());
            } else if root_dir.join("src/main.rs").exists() {
                bins.push(pkg.name.clone());
            }
        }

        return Ok((pkg.name.clone(), bins));
    }

    if let Some(ws) = &parsed.workspace {
        let members = ws.members.clone().unwrap_or_default();
        let ws_name = root_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string();
        return Ok((ws_name, members));
    }

    bail!("Cargo.toml contains neither [package] nor [workspace]")
}

pub fn compute_source_hash(root_dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let lock_file = root_dir.join("Cargo.lock");
    if let Ok(content) = fs::read(&lock_file) {
        hasher.update(&content);
    }

    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_entry(should_hash_entry)
        .flatten()
    {
        if entry.file_type().is_file() {
            hash_file_if_source(entry.path(), &mut hasher);
        }
    }

    Ok(format!("{:x}", hasher.finalize())[..16].to_string())
}

fn hash_file_if_source(path: &Path, hasher: &mut Sha256) {
    let is_source = path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext == "rs" || ext == "toml");

    if !is_source {
        return;
    }

    if let Ok(bytes) = fs::read(path) {
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update(&bytes);
    }
}

fn should_hash_entry(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    if name == "target" || name == ".git" || name == ".agents" || name == ".gemini" {
        return false;
    }
    true
}
