use anyhow::{Context, Result};

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

pub fn sanitize_filename(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    // Replace spaces and underscores with hyphens, remove any other non-alphanumeric chars
    let sanitized = stem
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c == ' ' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>();

    // Remove consecutive hyphens
    let sanitized = sanitized
        .split("--")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // Remove leading/trailing hyphens
    let sanitized = sanitized.trim_matches('-');

    // If somehow we end up with an empty string, use a default
    let sanitized = if sanitized.is_empty() {
        "output"
    } else {
        sanitized
    };

    path.with_file_name(format!("{}.jef", sanitized))
}

fn print_progress_dots(last_dot: Instant, dot_interval: Duration) -> Instant {
    let mut stdout = io::stdout();
    if last_dot.elapsed() >= dot_interval {
        print!(".");
        stdout.flush().unwrap_or_default();
        Instant::now()
    } else {
        last_dot
    }
}

pub fn wait_with_progress(
    child: &mut Child,
    dot_interval: Duration,
    poll_interval: Duration,
) -> io::Result<()> {
    let mut last_dot = Instant::now();

    while child.try_wait()?.is_none() {
        last_dot = print_progress_dots(last_dot, dot_interval);
        std::thread::sleep(poll_interval);
    }

    Ok(())
}

pub fn get_column_index(headers: &csv::StringRecord, name: &str) -> Result<usize> {
    headers
        .iter()
        .position(|h| h == name)
        .context(format!("Missing required column: {}", name))
}
