use anyhow::{Context, Result, bail};
use indicatif::ProgressBar;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn retrieve_binary(
    codespace_name: &str,
    remote_path: &str,
    bin_name: &str,
    dest_dir: &Path,
    install_to_bin: bool,
    spinner: &ProgressBar,
) -> Result<(PathBuf, u64)> {
    spinner.set_message(format!("Streaming compiled {bin_name} to local host..."));
    fs::create_dir_all(dest_dir).context("Failed to create destination output directory")?;

    let local_dest = dest_dir.join(bin_name);
    let mut file = fs::File::create(&local_dest)
        .with_context(|| format!("Failed to create local file {}", local_dest.display()))?;

    let cat_cmd = format!("cat '{remote_path}'");
    let output = Command::new("gh")
        .args(["codespace", "ssh", "-c", codespace_name, "--", "bash", "-c", &cat_cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to stream binary from remote Codespace")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to download compiled binary: {err}");
    }

    std::io::copy(&mut &output.stdout[..], &mut file)
        .context("Failed to write downloaded binary to disk")?;

    // chmod +x
    let mut perms = fs::metadata(&local_dest)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&local_dest, perms)?;

    let size = fs::metadata(&local_dest)?.len();

    if install_to_bin {
        if let Some(home) = dirs_home() {
            let bin_dir = home.join(".local/bin");
            fs::create_dir_all(&bin_dir)?;
            let target_link = bin_dir.join(bin_name);
            fs::copy(&local_dest, &target_link)?;
        }
    }

    Ok((local_dest, size))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
