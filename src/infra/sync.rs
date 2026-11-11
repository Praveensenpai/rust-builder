use anyhow::{Context, Result, bail};
use indicatif::ProgressBar;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::domain::config::REMOTE_PROJECTS_DIR;

pub fn sync_project_source(
    codespace_name: &str,
    project_name: &str,
    local_dir: &Path,
    spinner: &ProgressBar,
) -> Result<String> {
    spinner.set_message("Packaging and streaming project source over SSH...");
    let remote_dir = format!("{REMOTE_PROJECTS_DIR}/{project_name}");

    // Spawn local tar pack
    let mut tar_pack = Command::new("tar")
        .args([
            "-czf",
            "-",
            "--exclude=target",
            "--exclude=.git",
            "--exclude=.agents",
            "--exclude=.gemini",
            "-C",
            &local_dir.to_string_lossy(),
            ".",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn local tar process")?;

    let tar_out = tar_pack
        .stdout
        .take()
        .context("Failed to capture local tar stdout")?;

    // Remote extraction command
    let remote_cmd = format!("mkdir -p '{remote_dir}' && tar -xzf - -C '{remote_dir}'");

    let remote_proc = Command::new("gh")
        .args(["codespace", "ssh", "-c", codespace_name, "--", "bash", "-c", &remote_cmd])
        .stdin(tar_out)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to execute remote tar extraction via gh codespace ssh")?;

    let pack_res = tar_pack.wait().context("Tar packing process failed")?;
    if !pack_res.success() || !remote_proc.status.success() {
        let err = String::from_utf8_lossy(&remote_proc.stderr);
        bail!("Failed to stream source to Codespace: {err}");
    }

    Ok(remote_dir)
}
