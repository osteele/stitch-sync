use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub watch_dir: Option<PathBuf>,
    pub machine: Option<String>,
}
