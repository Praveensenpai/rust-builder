use anyhow::{Context, Result};
use console::style;
use std::process::Command;

const REPO: &str = "Praveensenpai/rust-builder";
const BINARY: &str = "rbuild";

pub fn handle_update() -> Result<()> {
    println!(
        "{} {}",
        style("rbuild").bold().cyan(),
        style("self-update").bright().black()
    );

    let current_version = env!("CARGO_PKG_VERSION");
    println!("  Current version: {}", style(current_version).cyan());

    // 1. Query latest release tag
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let output = Command::new("curl")
        .args(["-fsSL", &api_url])
        .output()
        .context("failed to run curl")?;
    if !output.status.success() {
        anyhow::bail!("failed to query latest release from GitHub");
    }
    let body = String::from_utf8_lossy(&output.stdout);
    let latest_tag = body
        .lines()
        .find(|l| l.contains("\"tag_name\""))
        .and_then(|l| l.split('"').nth(3))
        .ok_or_else(|| anyhow::anyhow!("could not parse latest tag from release API"))?;

    let latest_version = latest_tag.trim_start_matches('v');
    println!("  Latest version:  {}", style(latest_version).cyan());

    if latest_version == current_version {
        println!("{}", style("✔ Already up to date.").green());
        return Ok(());
    }

    let asset = format!("{}-x86_64-linux.tar.gz", BINARY);
    let base_url = format!(
        "https://github.com/{}/releases/download/{}",
        REPO, latest_tag
    );
    let download_url = format!("{}/{}", base_url, asset);
    let checksum_url = format!("{}/{}.sha256", base_url, asset);

    // 2. Download to temp dir
    let tmp_dir = std::env::temp_dir().join(format!("rbuild_update_{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).context("failed to create temp dir")?;

    let archive_path = tmp_dir.join(&asset);
    let checksum_path = tmp_dir.join(format!("{}.sha256", asset));

    println!("  Downloading {}...", style(&asset).cyan());
    let status = Command::new("curl")
        .args(["-fsSL", &download_url, "-o", archive_path.to_str().unwrap()])
        .status()
        .context("failed to run curl")?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        anyhow::bail!("failed to download release asset");
    }

    // 3. Verify checksum (best effort; fail if checksum exists but mismatch)
    let checksum_ok = Command::new("curl")
        .args([
            "-fsSL",
            &checksum_url,
            "-o",
            checksum_path.to_str().unwrap(),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if checksum_ok {
        let expected = std::fs::read_to_string(&checksum_path)
            .ok()
            .and_then(|s| s.split_whitespace().next().map(|x| x.to_string()));
        let actual = sha256_file(&archive_path)?;
        if let (Some(exp), Some(act)) = (expected, actual) {
            if exp != act {
                let _ = std::fs::remove_dir_all(&tmp_dir);
                anyhow::bail!("checksum mismatch: expected {} got {}", exp, act);
            }
            println!("{}", style("  ✔ Checksum verified.").green());
        }
    } else {
        println!(
            "{}",
            style("  ⚠ No checksum available; skipping verification.").yellow()
        );
    }

    // 4. Extract binary
    let status = Command::new("tar")
        .args([
            "-xzf",
            archive_path.to_str().unwrap(),
            "-C",
            tmp_dir.to_str().unwrap(),
        ])
        .status()
        .context("failed to run tar")?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        anyhow::bail!("failed to extract release archive");
    }

    let extracted = tmp_dir.join(BINARY);
    if !extracted.is_file() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        anyhow::bail!("extracted binary not found");
    }

    // 5. Replace current binary
    let target = std::env::current_exe().context("failed to get current executable path")?;
    let target_dir = target
        .parent()
        .ok_or_else(|| anyhow::anyhow!("no parent dir for current binary"))?;

    // If not writable, fall back to ~/.local/bin
    let install_path = if target_dir
        .metadata()
        .map(|m| !m.permissions().readonly())
        .unwrap_or(false)
    {
        target.clone()
    } else {
        let home = std::env::var("HOME").context("HOME not set")?;
        let bin_dir = std::path::Path::new(&home).join(".local/bin");
        std::fs::create_dir_all(&bin_dir).context("failed to create ~/.local/bin")?;
        bin_dir.join(BINARY)
    };

    // Write to a temp file in the same dir, then rename over the target.
    // Direct overwrite fails with ETXTBSY on a running binary.
    let tmp_bin = install_path.with_extension("new");
    std::fs::copy(&extracted, &tmp_bin)
        .with_context(|| format!("failed to stage binary at {}", tmp_bin.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_bin, std::fs::Permissions::from_mode(0o755))
            .context("failed to set permissions")?;
    }
    std::fs::rename(&tmp_bin, &install_path)
        .with_context(|| format!("failed to replace binary at {}", install_path.display()))?;

    let _ = std::fs::remove_dir_all(&tmp_dir);

    println!(
        "{}",
        style(format!(
            "✔ Updated rbuild to {} at {}",
            latest_version,
            install_path.display()
        ))
        .green()
        .bold()
    );
    println!(
        "  Restart your shell or run: {}",
        style("rbuild --version").cyan()
    );

    Ok(())
}

fn sha256_file(path: &std::path::Path) -> Result<Option<String>> {
    let output = Command::new("sha256sum").arg(path).output();
    if let Ok(out) = output {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            return Ok(s.split_whitespace().next().map(|x| x.to_string()));
        }
    }
    // fallback to shasum
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            return Ok(s.split_whitespace().next().map(|x| x.to_string()));
        }
    }
    Ok(None)
}
