mod file_conversion;
mod file_formats;
mod inkscape;
mod machines;
mod usb_drive;
mod utils;
mod watch;

use clap::Parser;

use std::path::PathBuf;

use crate::file_formats::FILE_FORMATS;
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
    },
    /// Show detailed information for a specific machine
    Info {
        /// Name of the machine
        name: String,
    },
}

fn list_machines_command(format: Option<String>) {
    let machines = machines::MACHINES.iter().filter(|machine| {
        format.as_ref().map_or(true, |f| {
            machine
                .formats
                .iter()
                .any(|fmt| fmt.extension.eq_ignore_ascii_case(f))
        })
    });

    for machine in machines {
        println!(
            "{} ({})",
            machine.name,
            machine
                .formats
                .iter()
                .map(|f| f.extension)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

fn watch_command(
    watch_dir: Option<PathBuf>,
    output_format: Option<String>,
    machine_name: Option<String>,
) {
    let copy_target_dir = find_embf_directory();
    let machine = machine_name
        .clone()
        .and_then(|m| machines::get_machine_info(&m));
    if machine_name.is_some() && machine.is_none() {
        println!("Machine '{}' not found", machine_name.unwrap());
        return;
    }
    watch::watch(watch_dir, copy_target_dir, output_format, &machine);
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
            MachineCommand::List { format } => list_machines_command(format),
            MachineCommand::Info { name } => match machines::get_machine_info(&name) {
                Some(info) => {
                    println!("{}", info.name);
                    println!(
                        "  Formats: {}",
                        info.formats
                            .iter()
                            .map(|f| f.extension)
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
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
        Commands::Machines { format } => list_machines_command(format),
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
