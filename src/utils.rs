use std::path::{Path, PathBuf};

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
