use anyhow::Result;
use colored::*;
use reqwest;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use crate::commands;
use crate::config::defaults::DEFAULT_FORMAT;
use crate::config::ConfigManager;
use crate::print_error;
use crate::print_notice;
use crate::services;
use crate::services::find_usb_containing_path;
use crate::services::inkscape;
use crate::services::Inkscape;
use crate::types::Machine;
use crate::types::FILE_FORMATS;
use crate::types::MACHINES;
use crate::utils;
use crate::utils::version;
use crate::Commands;
use crate::ConfigCommand;
use crate::ConfigKey;
use crate::MachineCommand;

impl Commands {
    pub fn execute(self) -> Result<()> {
        match self {
            Commands::Watch {
                dir,
                output_format,
                machine,
            } => commands::watch_command(dir, output_format, machine),
            Commands::Set { what, value } => {
                if what == "machine" {
                    ConfigCommand::Set {
                        key: ConfigKey::Machine,
                        value,
                    }
                    .execute()
                } else {
                    println!(
                        "Unknown setting: {}. Currently only 'machine' is supported.",
                        what
                    );
                    Ok(())
                }
            }
            Commands::Machine { command } => command.execute(),
            Commands::Machines { format, verbose } => {
                commands::list_machines_command(format, verbose)
            }
            Commands::Formats => Self::list_formats(),
            Commands::Config { command } => command.execute(),
            Commands::Update { dry_run } => commands::update_command(dry_run),
        }
    }

    fn list_formats() -> Result<()> {
        let mut formats = FILE_FORMATS.to_vec();
        formats.sort_by_key(|format| format.extension.to_owned());

        for format in formats {
            print!("{}: {}", format.extension, format.manufacturer);
            if let Some(notes) = format.notes {
                print!(" -- {}", notes);
            }
            println!();
        }
        Ok(())
    }
}

impl ConfigCommand {
    pub fn execute(self) -> Result<()> {
        let config_manager = ConfigManager::new()?;
        match self {
            ConfigCommand::Show => {
                let config = config_manager.load()?;
                if let Some(dir) = &config.watch_dir {
                    println!("Watch directory: {}", dir.display());
                }
                if let Some(machine) = &config.machine {
                    println!("Default machine: {}", machine);
                }
                Ok(())
            }
            ConfigCommand::Set { key, value } => match key {
                ConfigKey::WatchDir => {
                    let path = PathBuf::from(value.expect("Watch directory path is required"));
                    config_manager.set_watch_dir(path)?;
                    println!("Watch directory set");
                    Ok(())
                }
                ConfigKey::Machine => {
                    let machine = Self::select_machine(value);
                    if let Some(machine) = machine {
                        config_manager.set_machine(machine.name)?;
                        println!("Default machine set");
                    } else {
                        println!("No machine selected");
                    }
                    Ok(())
                }
            },
            ConfigCommand::Clear { key } => match key {
                ConfigKey::WatchDir => {
                    config_manager.clear_watch_dir()?;
                    println!("Watch directory cleared");
                    Ok(())
                }
                ConfigKey::Machine => {
                    config_manager.clear_machine()?;
                    println!("Default machine cleared");
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
    pub fn execute(self) -> Result<()> {
        match self {
            MachineCommand::List { format, verbose } => {
                commands::list_machines_command(format, verbose)
            }
            MachineCommand::Info { name } => Self::show_info(name),
        }
    }

    fn show_info(name: String) -> Result<()> {
        match Machine::interactive_find_by_name(&name) {
            Some(info) => {
                println!("{}", info.name);
                if let Some(notes) = &info.notes {
                    println!("  Notes: {}", notes);
                }
                if !info.synonyms.is_empty() {
                    println!("  Synonyms: {}", info.synonyms.join(", "));
                }
                if !info.formats.is_empty() {
                    println!("  Formats: {}", info.formats.join(", "));
                }
                if let Some(design_size) = &info.design_size {
                    println!("  Design size: {}", design_size);
                }
                if let Some(path) = &info.usb_path {
                    println!("  USB path: {}", path);
                }
            }
            None => println!("Machine '{}' not found", name),
        }
        Ok(())
    }
}

fn list_machines_command(format: Option<String>, verbose: bool) -> Result<()> {
    let machines = if let Some(format) = format {
        MACHINES
            .iter()
            .filter(|m| m.formats.contains(&format.to_lowercase()))
            .collect::<Vec<_>>()
    } else {
        MACHINES.iter().collect()
    };

    for machine in machines {
        if verbose {
            println!("{}", machine.name.bold());
            if !machine.synonyms.is_empty() {
                println!("  {} {}", "Synonyms:".blue(), machine.synonyms.join(", "));
            }
            if let Some(notes) = &machine.notes {
                println!("  {}: {}", "Note".blue(), notes);
            }
            if let Some(design_size) = &machine.design_size {
                println!("  {}: {}", "Design size".blue(), design_size);
            }
            if let Some(usb_path) = &machine.usb_path {
                println!("  {}: {}", "USB path".blue(), usb_path);
            }
        } else {
            println!("{} ({})", machine.name.bold(), machine.formats.join(", "));
        }
    }
    Ok(())
}

fn watch_command(
    watch_dir: Option<PathBuf>,
    output_format: Option<String>,
    machine_name: Option<String>,
) -> Result<()> {
    // Check for updates, but use cache
    if let Ok(Some(latest_version)) = version::get_latest_version(false) {
        print_notice!(
            "{} {} is available.",
            "A new version of stitch-sync",
            format!("({})", latest_version).dim()
        );
        println!(
            " → Run '{}' to upgrade.",
            "stitch-sync update".bright_green()
        );
    }

    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    let inkscape = Inkscape::find_app();
    if inkscape.is_none() {
        println!(
            "Inkscape not found. Please download and install from {}",
            inkscape::INKSCAPE_DOWNLOAD_URL
        );
        println!("Opening download page in your browser...");
        services::open_browser(inkscape::INKSCAPE_DOWNLOAD_URL);
        return Ok(());
    }
    if !inkscape.as_ref().unwrap().has_inkstitch {
        print_error!(
            "Warning: ink/stitch extension not found. Please install from {}",
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
        println!("Machine '{}' not found", machine_name.unwrap());
        return Ok(());
    }

    let usb_target_path = machine
        .as_ref()
        .and_then(|m| m.usb_path.as_deref())
        .unwrap_or_default();
    if let Some(usb_target_dir) = find_usb_containing_path(usb_target_path) {
        println!("USB target directory: {}", usb_target_dir.display());
    }

    // Determine accepted formats and preferred format
    let (accepted_formats, preferred_format) = match &machine {
        Some(machine) => {
            let formats = machine.formats.clone();
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
        println!("{} {}", "🤖 Machine:".bright_blue(), machine.name.bold());
    }
    println!(
        "{} {}",
        "Directory:".bright_blue(),
        watch_dir.display().to_string().bold()
    );
    match accepted_formats.len() {
        1 => println!(
            " {} {}",
            "→ Files will be converted to".bright_blue(),
            accepted_formats[0].bold()
        ),
        _ => println!(
            " {} {}",
            "→ Files will be converted to one of:".bright_blue(),
            accepted_formats.join(", ").bold()
        ),
    }
    println!(
        " {} {} {}",
        "→ Files will be copied to".bright_blue(),
        machine
            .as_ref()
            .and_then(|m| m.usb_path.as_deref())
            .unwrap_or("the root directory")
            .bold(),
        "on a mounted USB drive".bright_blue()
    );
    println!("\n{}", "Press 'q' to quit".bright_black().italic());

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

fn update_command(dry_run: bool) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    // Force fresh check for updates
    println!("Checking for updates...");
    let latest_version = match version::get_latest_version(true)? {
        Some(version) => version,
        None => {
            println!("You're already running the latest version!");
            return Ok(());
        }
    };

    println!("New version available: {}", latest_version);

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
    println!("Downloading new version...");
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
    println!("Extracting update...");
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
        println!("Dry run - not installing update");
        return Ok(());
    }

    // Replace current executable
    println!("Installing update...");
    let new_exe = tmp_dir.path().join(exe_name);
    fs::rename(&new_exe, &current_exe)?;

    println!("Successfully updated to version {}", latest_version);
    Ok(())
}
