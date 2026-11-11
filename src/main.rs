mod cli;
mod domain;
mod infra;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use std::process::ExitCode;
use std::time::Instant;

use cli::{Cli, Commands};
use domain::config::{BuildConfig, DEFAULT_CODESPACE_NAME};
use domain::project::{ProjectManifest, discover_project};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            ui::print_error(&format!("{err:#}"));
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        return handle_subcommand(command);
    }

    let start_time = Instant::now();
    let manifest = discover_project(&cli.path)?;
    let bin_name = cli
        .bin
        .as_deref()
        .or_else(|| manifest.binaries.first().map(|s| s.as_str()))
        .unwrap_or(&manifest.name);

    ui::print_step("Project", &format!("{} (hash: {})", manifest.name, manifest.source_hash));
    build_and_fetch(&cli, &manifest, bin_name, start_time)
}

fn build_and_fetch(
    cli: &Cli,
    manifest: &ProjectManifest,
    bin_name: &str,
    start_time: Instant,
) -> Result<()> {
    let repo = get_default_repo()?;
    let cs_name = cli.codespace.clone().unwrap_or_else(|| DEFAULT_CODESPACE_NAME.to_string());

    let spinner = ui::create_spinner("Connecting to remote Codespace runner...");
    let cs = infra::ensure_builder_codespace(&cs_name, &repo, &spinner)?;

    infra::check_and_clean_storage(&cs.name, &spinner)?;
    let remote_dir = infra::sync_project_source(&cs.name, &manifest.name, &manifest.root_dir, &spinner)?;

    let config = create_build_config(cli, &cs.name, bin_name);
    let remote_bin = infra::execute_remote_build(&remote_dir, bin_name, &config, &spinner)?;

    let dest_dir = cli.out_dir.clone().unwrap_or_else(|| {
        let profile = if config.release { "release" } else { "debug" };
        manifest.root_dir.join("target/remote").join(profile)
    });

    let (local_bin, bytes) = infra::retrieve_binary(
        &cs.name,
        &remote_bin,
        bin_name,
        &dest_dir,
        config.install,
        &spinner,
    )?;

    spinner.finish_and_clear();
    report_completion(bin_name, &local_bin, bytes, start_time, config.install);
    Ok(())
}

fn create_build_config(cli: &Cli, codespace: &str, bin_name: &str) -> BuildConfig {
    BuildConfig {
        codespace: codespace.to_string(),
        target: cli.target.clone(),
        release: !cli.debug,
        features: cli.features.clone(),
        all_features: cli.all_features,
        no_default_features: cli.no_default_features,
        bin: Some(bin_name.to_string()),
        package: cli.package.clone(),
        clean: cli.clean,
        force: cli.force,
        install: cli.install,
    }
}

fn report_completion(
    bin_name: &str,
    local_bin: &std::path::Path,
    bytes: u64,
    start_time: Instant,
    installed: bool,
) {
    let total_time = ui::format_duration(start_time.elapsed());
    let size_str = ui::format_bytes(bytes);

    ui::print_success(&format!(
        "Build complete: {} ({size_str}, {total_time})",
        style(bin_name).bold()
    ));
    ui::print_info("↳ Local binary:", &local_bin.display().to_string());
    if installed {
        ui::print_info("↳ Installed to:", &format!("~/.local/bin/{bin_name}"));
    }
}

fn handle_subcommand(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Status => cmd_status(),
        Commands::Stop { name } => cmd_stop(name),
        Commands::Ssh => cmd_ssh(),
        Commands::Clean => cmd_clean(),
    }
}

fn cmd_status() -> Result<()> {
    let list = infra::list_codespaces()?;
    if list.is_empty() {
        println!("No active Codespaces found.");
        return Ok(());
    }
    for item in list {
        let display = item.display_name.as_deref().unwrap_or(&item.name);
        println!(
            "  {} [{}] - state: {}",
            style(display).bold().cyan(),
            item.name,
            style(&item.state).green()
        );
    }
    Ok(())
}

fn cmd_stop(name: Option<String>) -> Result<()> {
    let target = name.unwrap_or_else(|| DEFAULT_CODESPACE_NAME.to_string());
    if let Some(cs) = infra::find_builder_codespace(&target)? {
        infra::stop_codespace(&cs.name)?;
        ui::print_success(&format!("Stopped Codespace {}", cs.name));
    } else {
        ui::print_error(&format!("Codespace {target} not found."));
    }
    Ok(())
}

fn cmd_ssh() -> Result<()> {
    let cs_name = DEFAULT_CODESPACE_NAME;
    if let Some(cs) = infra::find_builder_codespace(cs_name)? {
        let status = std::process::Command::new("gh")
            .args(["codespace", "ssh", "-c", &cs.name])
            .status()?;
        if !status.success() {
            ui::print_error("SSH session exited with error.");
        }
    } else {
        ui::print_error(&format!("Codespace {cs_name} not found."));
    }
    Ok(())
}

fn cmd_clean() -> Result<()> {
    let cs_name = DEFAULT_CODESPACE_NAME;
    if let Some(cs) = infra::find_builder_codespace(cs_name)? {
        let spinner = ui::create_spinner("Purging remote project caches...");
        infra::exec_ssh(&cs.name, "rm -rf /workspaces/rust-builder/projects/*/target")?;
        spinner.finish_and_clear();
        ui::print_success("Remote target caches cleaned.");
    }
    Ok(())
}

fn get_default_repo() -> Result<String> {
    let output = std::process::Command::new("gh")
        .args(["api", "user", "-q", ".login"])
        .output()
        .context("Failed to run 'gh api user'")?;

    if !output.status.success() {
        anyhow::bail!("GitHub CLI not authenticated. Please run 'gh auth login'.");
    }

    let login = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(format!("{login}/rust-builder"))
}
