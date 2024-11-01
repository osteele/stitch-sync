mod cli;
mod commands;
mod config;
mod services;
mod types;
mod utils;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use types::MACHINES;

use crate::cli::*;
use crate::commands::*;
use crate::config::ConfigManager;
use crate::types::Machine;
use crate::types::FILE_FORMATS;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Watch {
        dir: None,
        output_format: None,
        machine: None,
    }) {
        Commands::Watch {
            dir,
            output_format,
            machine,
        } => watch_command(dir, output_format, machine)?,
        Commands::Machine { command } => match command {
            MachineCommand::List { format, verbose } => list_machines_command(format, verbose)?,
            MachineCommand::Info { name } => match Machine::interactive_find_by_name(&name) {
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
            },
        },
        Commands::Machines { format, verbose } => list_machines_command(format, verbose)?,
        Commands::Formats => {
            let mut formats = FILE_FORMATS.to_vec();
            formats.sort_by_key(|format| format.extension.to_owned());

            for format in formats {
                print!("{}: {}", format.extension, format.manufacturer);
                if let Some(notes) = format.notes {
                    print!(" -- {}", notes);
                }
                println!();
            }
        }
        Commands::Config { command } => {
            let config_manager = ConfigManager::new()?;
            match command {
                ConfigCommand::Show => {
                    let config = config_manager.load()?;
                    if let Some(dir) = &config.watch_dir {
                        println!("Watch directory: {}", dir.display());
                    }
                    if let Some(machine) = &config.machine {
                        println!("Default machine: {}", machine);
                    }
                }
                ConfigCommand::Set { key, value } => match key {
                    ConfigKey::WatchDir => {
                        let path = PathBuf::from(value.expect("Watch directory path is required"));
                        config_manager.set_watch_dir(path)?;
                        println!("Watch directory set");
                    }
                    ConfigKey::Machine => {
                        let machine = if let Some(name) = value {
                            Machine::interactive_find_by_name(&name)
                        } else {
                            // Show list of all machines and let user choose
                            println!("Select your embroidery machine:");
                            let names: Vec<String> =
                                MACHINES.iter().map(|m| m.name.clone()).collect();
                            let index = utils::prompt_from_list(&names);
                            index.map(|i| MACHINES[i].clone())
                        };

                        if let Some(machine) = machine {
                            config_manager.set_machine(machine.name)?;
                            println!("Default machine set");
                        } else {
                            println!("No machine selected");
                        }
                    }
                },
                ConfigCommand::Clear { key } => match key {
                    ConfigKey::WatchDir => {
                        config_manager.clear_watch_dir()?;
                        println!("Watch directory cleared");
                    }
                    ConfigKey::Machine => {
                        config_manager.clear_machine()?;
                        println!("Default machine cleared");
                    }
                },
            }
        }
    }
    Ok(())
}
