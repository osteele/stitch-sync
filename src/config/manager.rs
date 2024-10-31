use super::types::Config;
use anyhow::{Context, Result};
use dirs::config_dir;
use std::fs;
use std::path::PathBuf;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = config_dir()
            .context("Could not determine config directory")?
            .join("stitch-sync");

        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");

        Ok(Self { config_path })
    }

    pub fn load(&self) -> Result<Config> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        toml::from_str(&content).context("Failed to parse config file")
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        let content = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn set_watch_dir(&self, path: PathBuf) -> Result<()> {
        let mut config = self.load()?;
        config.watch_dir = Some(path);
        self.save(&config)
    }

    pub fn set_machine(&self, machine: String) -> Result<()> {
        let mut config = self.load()?;
        config.machine = Some(machine);
        self.save(&config)
    }

    pub fn clear_watch_dir(&self) -> Result<()> {
        let mut config = self.load()?;
        config.watch_dir = None;
        self.save(&config)
    }

    pub fn clear_machine(&self) -> Result<()> {
        let mut config = self.load()?;
        config.machine = None;
        self.save(&config)
    }
}
