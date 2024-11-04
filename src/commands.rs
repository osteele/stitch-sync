use anyhow::Result;
use colored::*;

use std::path::PathBuf;

use crate::commands;
use crate::config::defaults::DEFAULT_FORMAT;
use crate::config::ConfigManager;
use crate::print_error;
use crate::services;
use crate::services::find_usb_containing_path;
use crate::services::inkscape;
use crate::services::Inkscape;
use crate::types::Machine;
use crate::types::FILE_FORMATS;
use crate::types::MACHINES;
use crate::utils;
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
            Commands::Machine { command } => command.execute(),
            Commands::Machines { format, verbose } => {
                commands::list_machines_command(format, verbose)
            }
            Commands::Formats => Self::list_formats(),
            Commands::Config { command } => command.execute(),
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
        println!("{} {}", "Machine:".blue(), machine.name.bold());
    }
    println!(
        "{} {}",
        "Watching directory:".blue(),
        watch_dir.display().to_string().bold()
    );
    match accepted_formats.len() {
        1 => println!(
            "{} {}",
            "Files will be converted to".blue(),
            accepted_formats[0].bold()
        ),
        _ => println!(
            "{} {}",
            "Files will be converted to one of the following formats:".blue(),
            accepted_formats.join(", ").bold()
        ),
    }
    println!(
        "{} {} {}",
        "Files will be copied to".blue(),
        machine
            .as_ref()
            .and_then(|m| m.usb_path.as_deref())
            .unwrap_or("the root directory")
            .bold(),
        "on a mounted USB drive".blue()
    );
    // let mut watch_formats = Vec::new();
    // watch_formats.extend(
    //     inkscape
    //         .unwrap()
    //         .supported_read_formats
    //         .iter()
    //         .map(|f| f.to_string()),
    // );
    // watch_formats.extend(accepted_formats.clone());
    // watch_formats.sort();
    // watch_formats.dedup();
    // println!("Watching for formats: {}", watch_formats.join(", "));

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
