use clap::Parser;
use clap::ValueEnum;

use std::path::PathBuf;

use crate::types::Machine;
use crate::types::MACHINES;
use crate::utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None, after_help = "\n\
\x1B[1;4mQuick Start Guide:\x1B[0m
  Run 'stitch-sync config set machine' to set your embroidery machine
  Run 'stitch-sync machine list' to see supported machines
  Run 'stitch-sync watch' to start watching for new designs

For more details, use --help with any command")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Parser)]
pub enum Commands {
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
pub enum MachineCommand {
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
pub enum ConfigCommand {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        #[arg(value_enum)]
        key: ConfigKey,
        /// Value to set (if not provided, will prompt for input)
        value: Option<String>,
    },
    /// Clear a configuration value
    Clear {
        #[arg(value_enum)]
        key: ConfigKey,
    },
}

impl ConfigCommand {
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

#[derive(Clone, ValueEnum)]
pub enum ConfigKey {
    /// Watch directory
    WatchDir,
    /// Default machine
    Machine,
}
