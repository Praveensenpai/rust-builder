use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.green} {msg}")
            .expect("Valid spinner template"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn print_step(title: &str, detail: &str) {
    println!("  {} {}", style(title).bold().cyan(), style(detail).dim());
}

pub fn print_success(msg: &str) {
    println!("  {} {}", style("✔").bold().green(), style(msg).bold());
}

pub fn print_error(msg: &str) {
    eprintln!("  {} {}", style("✖").bold().red(), style(msg).bold());
}

pub fn print_info(title: &str, content: &str) {
    println!("    {} {}", style(title).dim(), content);
}

pub fn format_duration(dur: Duration) -> String {
    let secs = dur.as_secs();
    if secs < 60 {
        format!("{:.1}s", dur.as_secs_f32())
    } else {
        let mins = secs / 60;
        let rem = secs % 60;
        format!("{mins}m {rem}s")
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
