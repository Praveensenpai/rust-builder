use anyhow::{Context, Result, bail};
use indicatif::ProgressBar;
use serde::Deserialize;
use std::process::Command;
use std::time::Duration;

use crate::domain::config::DEFAULT_IDLE_TIMEOUT;

#[derive(Debug, Clone, Deserialize)]
pub struct CodespaceItem {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub state: String,
}

pub fn list_codespaces() -> Result<Vec<CodespaceItem>> {
    let output = Command::new("gh")
        .args(["codespace", "list", "--json", "name,displayName,state,repository"])
        .output()
        .context("Failed to execute 'gh codespace list'")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to list codespaces: {err}");
    }

    let items: Vec<CodespaceItem> = serde_json::from_slice(&output.stdout)
        .context("Failed to parse codespace JSON list")?;

    Ok(items)
}

pub fn find_builder_codespace(target: &str) -> Result<Option<CodespaceItem>> {
    let items = list_codespaces()?;
    for item in items {
        if item.name == target {
            return Ok(Some(item));
        }
        if let Some(display) = &item.display_name {
            if display == target {
                return Ok(Some(item));
            }
        }
    }
    Ok(None)
}

pub fn ensure_builder_codespace(
    target_name: &str,
    repo: &str,
    spinner: &ProgressBar,
) -> Result<CodespaceItem> {
    if let Some(existing) = find_builder_codespace(target_name)? {
        if existing.state != "Available" {
            spinner.set_message(format!(
                "Waking up Codespace {} ({}) ...",
                existing.name, existing.state
            ));
            wake_codespace(&existing.name)?;
        }
        return Ok(existing);
    }

    spinner.set_message(format!(
        "Spinning up dedicated Codespace for {repo} (idle timeout: {DEFAULT_IDLE_TIMEOUT}) ..."
    ));

    create_codespace(repo, target_name)?;

    // Poll until available
    for _ in 0..30 {
        std::thread::sleep(Duration::from_secs(3));
        if let Some(cs) = find_builder_codespace(target_name)? {
            if cs.state == "Available" {
                return Ok(cs);
            }
            spinner.set_message(format!("Codespace starting: {} ...", cs.state));
        }
    }

    bail!("Timed out waiting for new Codespace to reach Available state")
}

fn create_codespace(repo: &str, display_name: &str) -> Result<()> {
    let status = Command::new("gh")
        .args([
            "codespace",
            "create",
            "-R",
            repo,
            "--display-name",
            display_name,
            "--idle-timeout",
            DEFAULT_IDLE_TIMEOUT,
        ])
        .status()
        .context("Failed to execute 'gh codespace create'")?;

    if !status.success() {
        bail!("'gh codespace create' exited with error status");
    }

    Ok(())
}

fn wake_codespace(name: &str) -> Result<()> {
    // A quick SSH check automatically starts a stopped codespace
    let _ = Command::new("gh")
        .args(["codespace", "ssh", "-c", name, "--", "echo", "awake"])
        .output();
    Ok(())
}

pub fn exec_ssh(name: &str, command: &str) -> Result<String> {
    let output = Command::new("gh")
        .args(["codespace", "ssh", "-c", name, "--", command])
        .output()
        .context("Failed to execute command via gh codespace ssh")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("Remote command failed: {err}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn stop_codespace(name: &str) -> Result<()> {
    let status = Command::new("gh")
        .args(["codespace", "stop", "-c", name])
        .status()
        .context("Failed to run 'gh codespace stop'")?;

    if !status.success() {
        bail!("Failed to stop codespace {name}");
    }

    Ok(())
}
