use anyhow::{Context, Result};
use indicatif::ProgressBar;
use std::process::Command;

use crate::domain::config::STORAGE_CLEANUP_THRESHOLD_PERCENT;

pub fn check_and_clean_storage(codespace_name: &str, spinner: &ProgressBar) -> Result<()> {
    spinner.set_message("Checking remote Codespace disk capacity...");

    let script = format!(
        r#"
USAGE=$(df /workspaces 2>/dev/null | awk 'NR==2 {{gsub("%","",$5); print $5}}')
if [ -z "$USAGE" ]; then
    USAGE=$(df / | awk 'NR==2 {{gsub("%","",$5); print $5}}')
fi
if [ -n "$USAGE" ] && [ "$USAGE" -ge {STORAGE_CLEANUP_THRESHOLD_PERCENT} ]; then
    echo "CLEANUP_TRIGGERED: $USAGE"
    find /workspaces/rust-builder/projects -maxdepth 3 -type d -name "target" -printf '%T@ %p\n' 2>/dev/null \
        | sort -n | head -n 3 | cut -d' ' -f2- | while read -r d; do
            rm -rf "$d"
        done
fi
echo "CAPACITY_OK: $USAGE"
"#
    );

    let output = Command::new("gh")
        .args(["codespace", "ssh", "-c", codespace_name, "--", &script])
        .output()
        .context("Failed to check remote disk capacity")?;

    let out_str = String::from_utf8_lossy(&output.stdout);
    if out_str.contains("CLEANUP_TRIGGERED") {
        spinner.set_message("Storage was near capacity (>80%): purged oldest target caches.");
    }

    Ok(())
}
