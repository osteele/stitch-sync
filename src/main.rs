mod file_conversion;
mod file_formats;
mod inkscape;
mod machines;
mod usb_drive;
mod utils;
mod watch;

use clap::Parser;
use crossterm::cursor;
use crossterm::execute;
use ctrlc;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use watch::watch_directory;
use watch::WatcherEvent;

use std::io::{self};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;

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
        #[arg(short, long, default_value = "jef")]
        output_format: String,
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

#[allow(dead_code)]
fn select_copy_target_directory() -> Option<PathBuf> {
    // Switch to normal mode
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Show).unwrap();

    println!("Please enter the path to the target directory:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let path = PathBuf::from(input.trim());

    if path.exists() && path.is_dir() {
        Some(path)
    } else {
        println!("Invalid directory. Please try again.");
        None
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Watch {
        dir: None,
        output_format: "jef".to_string(),
    }) {
        Commands::Watch { dir, output_format } => watch_command(dir, output_format),
        Commands::Machine { command } => match command {
            MachineCommand::List { format } => {
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
        Commands::Machines { format } => {
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

// Move the existing main functionality into this function
fn watch_command(dir: Option<PathBuf>, output_format: String) {
    // Set up signal handlers
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let inkscape_info = match inkscape::find_inkscape() {
        Some(info) => info,
        None => {
            println!("Inkscape not found. Please download and install from https://inkscape.org/release/1.4/mac-os-x/");
            return;
        }
    };

    let watch_dir = dir.unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join("Downloads")
    });

    if !watch_dir.exists() {
        println!("Directory does not exist: {}", watch_dir.display());
        return;
    }

    println!(
        "Setting up file watcher for directory: {}",
        watch_dir.display()
    );

    let (fs_tx, rx) = channel();

    // Create watcher with simplified event sending
    let mut watcher = match RecommendedWatcher::new(
        move |res| {
            if let Err(e) = fs_tx.send(WatcherEvent::File(res)) {
                eprintln!("Error sending event through channel: {:?}", e);
            }
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to create watcher: {:?}", e);
            return;
        }
    };

    // Set up watching with error handling
    match watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to watch directory: {:?}", e);
            return;
        }
    };

    let embf_dir = find_embf_directory();

    watch_directory(watch_dir, rx, inkscape_info, embf_dir, output_format);
    println!("File watcher stopped.");
}
