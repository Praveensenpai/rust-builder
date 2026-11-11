use anyhow::{Context, Result, bail};
use indicatif::ProgressBar;
use std::process::{Command, Stdio};

use crate::domain::config::BuildConfig;

pub fn execute_remote_build(
    remote_dir: &str,
    bin_name: &str,
    config: &BuildConfig,
    spinner: &ProgressBar,
) -> Result<String> {
    spinner.set_message(format!("Compiling {bin_name} on remote 4-vCPU runner..."));

    let args = build_cargo_args(bin_name, config);
    let joined_args = args.join(" ");
    let clean_step = if config.clean || config.force {
        "cargo clean && "
    } else {
        ""
    };

    let remote_cmd = format!(
        "export RUSTC_WRAPPER=sccache && cd '{remote_dir}' && {clean_step}cargo {joined_args}"
    );

    let output = Command::new("gh")
        .args(["codespace", "ssh", "-c", &config.codespace, "--", &remote_cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run remote cargo build via gh codespace ssh")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!("Remote build failed:\n{stderr}\n{stdout}");
    }

    Ok(resolve_remote_binary_path(remote_dir, bin_name, config))
}

fn build_cargo_args(bin_name: &str, config: &BuildConfig) -> Vec<String> {
    let mut args = vec!["build".to_string()];
    if config.release {
        args.push("--release".to_string());
    }
    if let Some(p) = &config.package {
        args.push(format!("-p {p}"));
    }
    if let Some(b) = &config.bin {
        args.push(format!("--bin {b}"));
    } else if !bin_name.is_empty() {
        args.push(format!("--bin {bin_name}"));
    }
    if let Some(t) = &config.target {
        args.push(format!("--target {t}"));
    }
    if let Some(f) = &config.features {
        args.push(format!("--features {f}"));
    }
    if config.all_features {
        args.push("--all-features".to_string());
    }
    if config.no_default_features {
        args.push("--no-default-features".to_string());
    }
    args
}

fn resolve_remote_binary_path(
    remote_dir: &str,
    bin_name: &str,
    config: &BuildConfig,
) -> String {
    let profile = if config.release { "release" } else { "debug" };
    match config.target.as_deref() {
        Some(t) if !t.is_empty() => {
            format!("{remote_dir}/target/{t}/{profile}/{bin_name}")
        }
        _ => format!("{remote_dir}/target/{profile}/{bin_name}"),
    }
}
