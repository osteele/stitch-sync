use anyhow::{Context, Result};


pub fn get_column_index(headers: &csv::StringRecord, name: &str) -> Result<usize> {
    headers
        .iter()
        .position(|h| h == name)
        .context(format!("Missing required column: {}", name))
}
