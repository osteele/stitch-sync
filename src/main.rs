mod config;
mod services;
mod types;
mod usb_drive;
mod utils;

use clap::Parser;

use std::path::PathBuf;

use crate::config::defaults::DEFAULT_FORMAT;
use crate::types::machine;
use crate::types::FILE_FORMATS;
use crate::types::MACHINES;
use crate::usb_drive::find_embf_directory;

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

fn list_machines_command(format: Option<String>, verbose: bool) {
    let machines = MACHINES.iter().filter(|machine| {
        format.as_ref().map_or(true, |f| {
            machine
                .formats
                .iter()
                .any(|fmt| fmt.eq_ignore_ascii_case(f))
        })
    });

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
}

fn watch_command(
    watch_dir: Option<PathBuf>,
    output_format: Option<String>,
    machine_name: Option<String>,
) {
    let copy_target_dir = find_embf_directory();
    let machine = machine_name
        .as_ref()
        .and_then(|m| machine::get_machine_info(&m));
    if machine_name.is_some() && machine.is_none() {
        println!("Machine '{}' not found", machine_name.unwrap());
        return;
    }
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
}

fn main() {
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
        } => watch_command(dir, output_format, machine),
        Commands::Machine { command } => match command {
            MachineCommand::List { format, verbose } => list_machines_command(format, verbose),
            MachineCommand::Info { name } => match machine::get_machine_info(&name) {
                Some(info) => {
                    println!("{}", info.name);
                    println!("  Formats: {}", info.formats.join(", "));
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
        Commands::Machines { format, verbose } => list_machines_command(format, verbose),
        Commands::Formats => {
            let mut formats = FILE_FORMATS.to_vec();
            formats.sort_by_key(|format| format.extension);

            for format in formats {
                print!("{}: {}", format.extension, format.manufacturer);
                if let Some(notes) = format.notes {
                    print!(" -- {}", notes);
                }
                println!();
            }
        }
    }
}
