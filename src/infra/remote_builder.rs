use anyhow::{bail, Context, Result};
use indicatif::ProgressBar;
use std::process::{Command, Stdio};

use crate::domain::config::{BuildConfig, VerifyKind};

pub fn execute_remote_build(
    remote_dir: &str,
    bin_name: &str,
    config: &BuildConfig,
    spinner: &ProgressBar,
) -> Result<String> {
    spinner.set_message(format!("Compiling {bin_name} on remote 4-vCPU runner..."));

    let args = build_cargo_args(bin_name, config);
    let joined_args = join_shell_args(&args);
    let clean_step = if config.clean || config.force {
        "cargo clean && "
    } else {
        ""
    };

    let remote_cmd = format!(
        "export RUSTC_WRAPPER=sccache && cd '{remote_dir}' && {clean_step}cargo {joined_args}"
    );

    let output = Command::new("gh")
        .args([
            "codespace",
            "ssh",
            "-c",
            &config.codespace,
            "--",
            &remote_cmd,
        ])
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

pub fn execute_remote_verify(
    remote_dir: &str,
    kind: &VerifyKind,
    config: &BuildConfig,
    spinner: &ProgressBar,
) -> Result<()> {
    let subcommand = kind.cargo_subcommand();
    spinner.set_message(format!("Running remote cargo {subcommand}..."));

    let args = verify_cargo_args(kind, config);
    let joined_args = join_shell_args(&args);
    let remote_cmd =
        format!("export RUSTC_WRAPPER=sccache && cd '{remote_dir}' && cargo {joined_args}");

    let status = spinner.suspend(|| {
        Command::new("gh")
            .args([
                "codespace",
                "ssh",
                "-c",
                &config.codespace,
                "--",
                &remote_cmd,
            ])
            .status()
            .context("Failed to run remote cargo verification via gh codespace ssh")
    })?;

    if !status.success() {
        bail!(
            "Remote cargo {subcommand} failed with exit code {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn build_cargo_args(bin_name: &str, config: &BuildConfig) -> Vec<String> {
    let mut args = vec!["build".to_string()];
    if config.release {
        args.push("--release".to_string());
    }
    push_opt(&mut args, "-p", &config.package);
    if let Some(b) = &config.bin {
        push_pair(&mut args, "--bin", b);
    } else if !bin_name.is_empty() {
        push_pair(&mut args, "--bin", bin_name);
    }
    push_opt(&mut args, "--target", &config.target);
    push_opt(&mut args, "--features", &config.features);
    if config.all_features {
        args.push("--all-features".to_string());
    }
    if config.no_default_features {
        args.push("--no-default-features".to_string());
    }
    args
}

fn verify_cargo_args(kind: &VerifyKind, config: &BuildConfig) -> Vec<String> {
    let mut args = vec![kind.cargo_subcommand().to_string()];
    push_opt(&mut args, "-p", &config.package);
    push_opt(&mut args, "--target", &config.target);
    push_opt(&mut args, "--features", &config.features);
    if config.all_features {
        args.push("--all-features".to_string());
    }
    if config.no_default_features {
        args.push("--no-default-features".to_string());
    }
    args.push("--all-targets".to_string());
    if *kind == VerifyKind::Clippy {
        args.push("--".to_string());
        args.push("-D".to_string());
        args.push("warnings".to_string());
    }
    args
}

fn push_opt(args: &mut Vec<String>, flag: &str, value: &Option<String>) {
    if let Some(v) = value {
        push_pair(args, flag, v);
    }
}

fn push_pair(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.to_string());
    args.push(value.to_string());
}

fn join_shell_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| format!("'{}'", arg.replace('\'', "'\\''")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn resolve_remote_binary_path(remote_dir: &str, bin_name: &str, config: &BuildConfig) -> String {
    let profile = if config.release { "release" } else { "debug" };
    match config.target.as_deref() {
        Some(t) if !t.is_empty() => {
            format!("{remote_dir}/target/{t}/{profile}/{bin_name}")
        }
        _ => format!("{remote_dir}/target/{profile}/{bin_name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> BuildConfig {
        BuildConfig::default()
    }

    #[test]
    fn clippy_args_deny_warnings() {
        assert_eq!(
            verify_cargo_args(&VerifyKind::Clippy, &config()),
            vec!["clippy", "--all-targets", "--", "-D", "warnings"]
        );
    }

    #[test]
    fn test_args_cover_all_targets() {
        assert_eq!(
            verify_cargo_args(&VerifyKind::Test, &config()),
            vec!["test", "--all-targets"]
        );
    }

    #[test]
    fn shell_join_quotes_each_token() {
        assert_eq!(
            join_shell_args(&["-p".to_string(), "my pkg".to_string()]),
            "'-p' 'my pkg'"
        );
    }

    #[test]
    fn build_args_split_flag_values() {
        assert_eq!(
            build_cargo_args("tool", &config()),
            vec!["build", "--release", "--bin", "tool"]
        );
    }
}
