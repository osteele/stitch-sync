use anyhow::Result;
use colored::Colorize as Colorize;
use crossterm::style::Stylize;
use reqwest;

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;

use crate::config::defaults::DEFAULT_FORMAT;
use crate::config::ConfigManager;
use crate::print_error;
use crate::write_notice;
use crate::services;
use crate::services::find_usb_containing_path;
use crate::services::inkscape;
use crate::services::Inkscape;
use crate::types::Machine;
use crate::types::FILE_FORMATS;
use crate::types::MACHINES;
use crate::utils;
use crate::utils::version;

use super::{Commands, ConfigCommand, ConfigKey, MachineCommand};

impl Commands {
    pub fn execute<W: Write>(self, writer: &mut W) -> Result<()> {
        match self {
            Commands::Watch {
                dir,
                output_format,
                machine,
            } => watch_command(dir, output_format, machine, writer),
            Commands::Set { what, value } => {
                if what == "machine" {
                    ConfigCommand::Set {
                        key: ConfigKey::Machine,
                        value,
                    }
                    .execute(writer)
                } else {
                    writeln!(
                        writer,
                        "Unknown setting: {}. Currently only 'machine' is supported.",
                        what
                    )?;
                    Ok(())
                }
            }
            Commands::Machine { command } => command.execute(writer),
            Commands::Machines { format, verbose } => {
                list_machines_command(format, verbose, writer)
            }
            Commands::Formats => Self::list_formats(writer),
            Commands::Config { command } => command.execute(writer),
            Commands::Update { dry_run } => update_command(dry_run, writer),
        }
    }

    fn list_formats<W: Write>(writer: &mut W) -> Result<()> {
        let mut formats = FILE_FORMATS.to_vec();
        formats.sort_by_key(|format| format.extension.to_owned());

        for format in formats {
            write!(writer, "{}: {}", format.extension, format.manufacturer)?;
            if let Some(notes) = format.notes {
                write!(writer, " -- {}", notes)?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

impl ConfigCommand {
    pub fn execute<W: Write>(self, writer: &mut W) -> Result<()> {
        let config_manager = ConfigManager::new()?;
        match self {
            ConfigCommand::Show => {
                let config = config_manager.load()?;
                if let Some(dir) = &config.watch_dir {
                    writeln!(writer, "Watch directory: {}", dir.display())?;
                }
                if let Some(machine) = &config.machine {
                    writeln!(writer, "Default machine: {}", machine)?;
                }
                Ok(())
            }
            ConfigCommand::Set { key, value } => match key {
                ConfigKey::WatchDir => {
                    let path = PathBuf::from(value.expect("Watch directory path is required"));
                    config_manager.set_watch_dir(path)?;
                    writeln!(writer, "Watch directory set")?;
                    Ok(())
                }
                ConfigKey::Machine => {
                    let machine = Self::select_machine(value);
                    if let Some(machine) = machine {
                        config_manager.set_machine(machine.name)?;
                        writeln!(writer, "Default machine set")?;
                    } else {
                        writeln!(writer, "No machine selected")?;
                    }
                    Ok(())
                }
            },
            ConfigCommand::Clear { key } => match key {
                ConfigKey::WatchDir => {
                    config_manager.clear_watch_dir()?;
                    writeln!(writer, "Watch directory cleared")?;
                    Ok(())
                }
                ConfigKey::Machine => {
                    config_manager.clear_machine()?;
                    writeln!(writer, "Default machine cleared")?;
                    Ok(())
                }
            },
        }
    }

    pub fn select_machine(value: Option<String>) -> Option<Machine> {
        if let Some(name) = value {
            Machine::interactive_find_by_name(&name)
        } else {
            // Show list of all machines and let user choose
            println!("Select your embroidery machine:");
            let mut names: Vec<String> = MACHINES
                .iter()
                .flat_map(|m| {
                    let mut synonyms = m.synonyms.clone();
                    synonyms.push(m.name.clone());
                    synonyms
                })
                .filter(|n| !n.is_empty())
                .collect::<Vec<String>>();
            names.sort();
            let index = utils::prompt_from_list(&names);
            index.map(|i| MACHINES[i].clone())
        }
    }
}

impl MachineCommand {
    pub fn execute<W: Write>(self, writer: &mut W) -> Result<()> {
        match self {
            MachineCommand::List { format, verbose } => {
                list_machines_command(format, verbose, writer)
            }
            MachineCommand::Info { name } => Self::show_info(name, writer),
        }
    }

    fn show_info<W: Write>(name: String, writer: &mut W) -> Result<()> {
        match Machine::interactive_find_by_name(&name) {
            Some(info) => {
                writeln!(writer, "{}", info.name)?;
                if let Some(notes) = &info.notes {
                    writeln!(writer, "  Notes: {}", notes)?;
                }
                if !info.synonyms.is_empty() {
                    writeln!(writer, "  Synonyms: {}", info.synonyms.join(", "))?;
                }
                if !info.file_formats.is_empty() {
                    writeln!(writer, "  Formats: {}", info.file_formats.join(", "))?;
                }
                if let Some(design_size) = &info.design_size {
                    writeln!(writer, "  Design size: {}", design_size)?;
                }
                if let Some(path) = &info.usb_path {
                    writeln!(writer, "  USB path: {}", path)?;
                }
            }
            None => writeln!(writer, "Machine '{}' not found", name)?,
        }
        Ok(())
    }
}

fn list_machines_command<W: Write>(
    format: Option<String>,
    verbose: bool,
    writer: &mut W,
) -> Result<()> {
    let machines = if let Some(format) = format {
        MACHINES
            .iter()
            .filter(|m| m.file_formats.contains(&format.to_lowercase()))
            .collect::<Vec<_>>()
    } else {
        MACHINES.iter().collect()
    };

    for machine in machines {
        if verbose {
            writeln!(writer, "{}", machine.name.clone().bold())?;
            if !machine.synonyms.is_empty() {
                writeln!(writer, "  {} {}", "Synonyms:".stylize().blue(), machine.synonyms.join(", "))?;
            }
            if let Some(notes) = &machine.notes {
                writeln!(writer, "  {}: {}", "Note".stylize().blue(), notes)?;
            }
            if let Some(design_size) = &machine.design_size {
                writeln!(writer, "  {}: {}", "Design size".stylize().blue(), design_size)?;
            }
            if let Some(usb_path) = &machine.usb_path {
                writeln!(writer, "  {}: {}", "USB path".stylize().blue(), usb_path)?;
            }
        } else {
            writeln!(
                writer,
                "{} ({})",
                machine.name.clone().bold(),
                machine.file_formats.join(", ")
            )?;
        }
    }
    Ok(())
}

fn watch_command<W: Write>(
    watch_dir: Option<PathBuf>,
    output_format: Option<String>,
    machine_name: Option<String>,
    writer: &mut W,
) -> Result<()> {
    // Check for updates, but use cache
    if let Ok(Some(latest_version)) = version::get_latest_version(false) {
        write_notice!(writer, "🔄 A new version of stitch-sync {} is available.", format!("({})", latest_version).dim());
        writeln!(writer, " → Run '{}' to upgrade.", "stitch-sync update".bright_green())?;
    }

    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    let inkscape = Inkscape::find_app();
    if inkscape.is_none() {
        print_error!(
            "🚨 Inkscape not found. Please download and install from {}",
            inkscape::INKSCAPE_DOWNLOAD_URL
        );
        println!("Opening download page in your browser...");
        services::open_browser(inkscape::INKSCAPE_DOWNLOAD_URL);
        return Ok(());
    }
    if !inkscape.as_ref().unwrap().has_inkstitch {
        print_error!(
            "🚨 Warning: ink/stitch extension not found. Please install from {}",
            inkscape::INKSTITCH_INSTALL_URL
        );
    }

    let watch_dir = watch_dir.or(config.watch_dir).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join("Downloads")
    });

    let machine_name = machine_name.or(config.machine);
    let machine = machine_name
        .as_ref()
        .and_then(|m| Machine::interactive_find_by_name(m));
    if machine_name.is_some() && machine.is_none() {
        print_error!("🚨 Machine '{}' not found", machine_name.unwrap());
        return Ok(());
    }

    let usb_target_path = machine
        .as_ref()
        .and_then(|m| m.usb_path.as_deref())
        .unwrap_or_default();
    if let Some(usb_target_dir) = find_usb_containing_path(usb_target_path) {
        writeln!(writer, "💾 USB target directory: {}", usb_target_dir.display())?;
    }

    // Determine accepted formats and preferred format
    let (accepted_formats, preferred_format) = match &machine {
        Some(machine) => {
            let formats = machine.file_formats.clone();
            let preferred = output_format
                .or_else(|| formats.first().map(|s| s.to_string()))
                .unwrap_or_else(|| DEFAULT_FORMAT.to_string())
                .to_lowercase();
            (formats, preferred)
        }
        None => {
            let preferred = output_format.unwrap_or_else(|| DEFAULT_FORMAT.to_string());
            (vec![preferred.clone()], preferred)
        }
    };

    // Convert preferred format to 'jef' if it ends with 'jef+'
    let preferred_format = if preferred_format == "jef+"
        && !inkscape
            .as_ref()
            .unwrap()
            .supported_write_formats
            .contains(&preferred_format.as_str())
    {
        "jef".to_string()
    } else {
        preferred_format
    };

    if let Some(ref machine) = machine {
        writeln!(writer, "{} {}", "🧵 Machine:".bright_blue(), machine.name.clone().bold())?;
    }
    writeln!(writer, "{} {}", "📁 Watch directory:".bright_blue(), watch_dir.display().to_string().bold())?;
    match accepted_formats.len() {
        1 => writeln!(writer, " {} {}", "→ Files will be converted to".bright_blue(), accepted_formats[0].clone().bold())?,
        _ => writeln!(writer, " {} {}", "→ Files will be converted to one of:".bright_blue(), accepted_formats.join(", ").bold())?,
    }
    writeln!(writer, " {} {} {}", "→ Files will be copied into the".bright_blue(), machine
        .as_ref()
        .and_then(|m| m.usb_path.as_deref())
        .unwrap_or(" root ")
        .stylize().bold(),
        "directory on a mounted USB drive".bright_blue())?;
    writeln!(writer, "\n{}", "Press 'q' to quit".bright_black().italic())?;

    services::watch_dir(
        &watch_dir,
        &Some(usb_target_path),
        &accepted_formats
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        &preferred_format,
    );
    Ok(())
}

fn update_command<W: Write>(dry_run: bool, writer: &mut W) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    writeln!(writer, "Current version: {}", current_version)?;

    // Force fresh check for updates
    writeln!(writer, "Checking for updates...")?;
    let latest_version = match version::get_latest_version(true)? {
        Some(version) => version,
        None => {
            writeln!(writer, "You're already running the latest version!")?;
            return Ok(());
        }
    };

    writeln!(writer, "New version available: {}", latest_version)?;

    // Get platform-specific info
    let (platform, exe_name) = match std::env::consts::OS {
        "macos" => ("apple-darwin", "stitch-sync"),
        "linux" => ("unknown-linux-gnu", "stitch-sync"),
        "windows" => ("pc-windows-msvc", "stitch-sync.exe"),
        _ => return Err(anyhow::anyhow!("Unsupported platform")),
    };

    // Create temporary directory that will be cleaned up when we're done
    let tmp_dir = tempfile::tempdir()?;
    let _tmp_guard = scopeguard::guard(tmp_dir.path().to_path_buf(), |p| {
        let _ = fs::remove_dir_all(p);
    });

    // Download new version
    writeln!(writer, "⬇️  Downloading new version...")?;
    let asset_name = format!("stitch-sync-x86_64-{}.tar.gz", platform);
    let download_url = format!(
        "https://github.com/osteele/stitch-sync/releases/download/v{}/{}",
        latest_version, asset_name
    );

    let archive_path = tmp_dir.path().join(&asset_name);
    let client = reqwest::blocking::Client::new();
    let response = client.get(&download_url).send()?;
    let content = response.bytes()?;
    fs::write(&archive_path, content)?;

    // Extract archive
    writeln!(writer, "⬇️  Extracting update...")?;
    let output = process::Command::new("tar")
        .arg("xzf")
        .arg(&archive_path)
        .current_dir(tmp_dir.path())
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to extract archive"));
    }

    // Get current executable path
    let current_exe = env::current_exe()?;

    if dry_run {
        writeln!(writer, "Dry run - not installing update")?;
        return Ok(());
    }

    // Replace current executable
    writeln!(writer, "⬇️  Installing update...")?;
    let new_exe = tmp_dir.path().join(exe_name);
    fs::rename(&new_exe, &current_exe)?;

    writeln!(writer, "✅ Successfully updated to version {}", latest_version)?;
    Ok(())
}