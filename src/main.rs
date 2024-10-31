mod config;
mod services;
mod types;
mod usb_drive;
mod utils;

use clap::Parser;
use clap::ValueEnum;
use types::machine::Machine;

use std::path::PathBuf;

use crate::config::defaults::DEFAULT_FORMAT;
use crate::types::FILE_FORMATS;
use crate::types::MACHINES;
use crate::usb_drive::find_usb_containing_path;

use crate::config::ConfigManager;

use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    /// Watch directory and convert files
    Watch {
        /// Directory to watch for new DST files
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Output format (e.g., 'jef', 'pes')
        #[arg(short, long)]
        output_format: Option<String>,
        /// Target machine (determines accepted formats)
        #[arg(short, long)]
        machine: Option<String>,
    },
    /// Machine-related commands
    Machine {
        #[command(subcommand)]
        command: MachineCommand,
    },
    /// List all supported machines (alias for 'machine list')
    Machines {
        /// Filter by file format
        #[arg(short, long)]
        format: Option<String>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// List supported file formats
    Formats,
    /// Configuration commands
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Parser)]
enum MachineCommand {
    /// List all supported machines
    List {
        /// Filter by file format
        #[arg(short, long)]
        format: Option<String>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show detailed information for a specific machine
    Info {
        /// Name of the machine
        name: String,
    },
}

#[derive(Parser)]
enum ConfigCommand {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        #[arg(value_enum)]
        key: ConfigKey,
        value: String,
    },
    /// Clear a configuration value
    Clear {
        #[arg(value_enum)]
        key: ConfigKey,
    },
}

#[derive(Clone, ValueEnum)]
enum ConfigKey {
    /// Watch directory
    WatchDir,
    /// Default machine
    Machine,
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
        println!("{} ({})", machine.name, machine.formats.join(", "));
        if verbose {
            if let Some(notes) = &machine.notes {
                println!("  Note: {}", notes);
            }
            if let Some(design_size) = &machine.design_size {
                println!("  Design size: {}", design_size);
            }
            if let Some(usb_path) = &machine.usb_path {
                println!("  USB path: USB Drive:/{:?}", usb_path);
            }
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

    let watch_dir = watch_dir.or(config.watch_dir).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join("Downloads")
    });

    let machine_name = machine_name.or(config.machine);
    let machine = machine_name
        .as_ref()
        .and_then(|m| Machine::find_by_name(&m));
    if machine_name.is_some() && machine.is_none() {
        println!("Machine '{}' not found", machine_name.unwrap());
        return Ok(());
    }

    let copy_target_path = machine
        .as_ref()
        .map(|m| m.usb_path.as_deref())
        .flatten()
        .unwrap_or("");
    let copy_target_dir = find_usb_containing_path(&copy_target_path);

    // Determine accepted formats and preferred format
    let (accepted_formats, preferred_format) = match &machine {
        Some(machine) => {
            let formats = machine.formats.clone();
            let preferred = output_format
                .or_else(|| formats.first().map(|s| s.to_string()))
                .unwrap_or_else(|| DEFAULT_FORMAT.to_string());
            (formats, preferred)
        }
        None => {
            let preferred = output_format.unwrap_or_else(|| DEFAULT_FORMAT.to_string());
            (vec![preferred.clone()], preferred)
        }
    };
    services::watch_dir(
        watch_dir,
        copy_target_dir,
        accepted_formats,
        preferred_format,
    );
    Ok(())
}

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
            MachineCommand::Info { name } => match Machine::find_by_name(&name) {
                Some(info) => {
                    println!("{}", info.name);
                    println!("  Formats: {}", info.formats.join(", "));
                    if let Some(notes) = &info.notes {
                        println!("  Notes: {}", notes);
                    }
                    if let Some(path) = &info.usb_path {
                        println!("  USB path: {}", path);
                    }
                    if let Some(notes) = &info.notes {
                        println!("  Notes: {}", notes);
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
                        let path = PathBuf::from(value);
                        config_manager.set_watch_dir(path)?;
                        println!("Watch directory set");
                    }
                    ConfigKey::Machine => {
                        if let Some(machine) = Machine::find_by_name(&value) {
                            config_manager.set_machine(machine.name)?;
                            println!("Default machine set");
                        } else {
                            println!("Machine '{}' not found", value);
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
