mod inkscape;
mod usb_drive;
mod utils;

use clap::Parser;
use crossterm::cursor;
use crossterm::execute;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ctrlc;
use notify::{Config, Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

use std::error::Error;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use crate::inkscape::InkscapeInfo;
use crate::usb_drive::{find_embf_directory, unmount_usb_volume};
use crate::utils::sanitize_filename;

/// Convert DST embroidery files to JEF format using Inkscape with ink/stitch
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to watch for new DST files
    #[arg(short, long)]
    dir: Option<PathBuf>,
}

#[allow(dead_code)]
fn select_copy_target_directory() -> Option<PathBuf> {
    // Switch to normal mode
    let mut stdout = io::stdout();
    disable_raw_mode().unwrap();
    execute!(stdout, cursor::Show).unwrap();

    println!("Please enter the path to the target directory:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let path = PathBuf::from(input.trim());

    // Switch back to raw mode
    enable_raw_mode().unwrap();
    execute!(stdout, cursor::Hide).unwrap();

    if path.exists() && path.is_dir() {
        Some(path)
    } else {
        println!("Invalid directory. Please try again.");
        None
    }
}

/// Handles a file creation event, converting DST to JEF and copying to target directory
fn handle_file_creation(
    path: &Path,
    inkscape_info: &InkscapeInfo,
    embf_dir: &Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();
    if path.extension() != Some(OsStr::new("dst")) {
        return Ok(());
    }

    disable_raw_mode().unwrap();

    println!("New DST file detected: {}", path.display());
    print!("Converting {} to JEF using Inkscape...", path.display());
    stdout.flush()?;
    let output_path = sanitize_filename(path);
    let start = Instant::now();
    let output = Command::new(&inkscape_info.path)
        .arg(path)
        .arg("--export-filename")
        .arg(&output_path)
        .output()?;
    println!();

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("extension not found") || error.contains("unknown extension") {
            println!("ink/stitch extension not installed or not working properly. Please download and install from https://inkstitch.org/docs/install/");
        } else {
            println!("Error converting file: {}", error);
        }
        enable_raw_mode().unwrap();
        return Ok(());
    }

    let elapsed = start.elapsed();
    println!(
        "Converted to JEF: {} ({:.2}s)",
        output_path.display(),
        elapsed.as_secs_f32()
    );

    if let Some(ref embf_dir) = embf_dir {
        let dest = embf_dir.join(output_path.file_name().unwrap());
        std::fs::copy(&output_path, &dest)?;
        println!("Copied to EMB directory: {}", dest.display());
    }

    // Return to raw mode when done
    // execute!(stdout, cursor::Hide).unwrap();
    enable_raw_mode().unwrap();
    // execute!(stdout, cursor::Hide).unwrap();

    Ok(())
}

// Returns true if the program should exit
fn handle_key_event(key: KeyEvent, embf_dir: &mut Option<PathBuf>) -> Result<bool, io::Error> {
    match (key.code, key.modifiers.contains(KeyModifiers::CONTROL)) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), true) => Ok(true),
        (KeyCode::Char('t'), _) => {
            if let Some(new_dir) = select_copy_target_directory() {
                *embf_dir = Some(new_dir);
                println!("New target directory set.");
            }
            Ok(false)
        }
        (KeyCode::Char('u'), _) => {
            unmount_usb_volume();
            Ok(false)
        }
        _ => Ok(false),
    }
}

// Remove the Keyboard variant as we'll handle keyboard events directly
#[derive(Debug)]
enum WatcherEvent {
    File(notify::Result<NotifyEvent>),
}

fn watch_directory(
    path: impl AsRef<Path>,
    event_rx: Receiver<WatcherEvent>,
    inkscape_info: InkscapeInfo,
    mut embf_dir: Option<PathBuf>,
) {
    let warn_inkstitch = false;

    if warn_inkstitch && !inkscape_info.has_inkstitch {
        println!("Warning: ink/stitch extension not found. Please install from https://inkstitch.org/docs/install/");
    }

    if let Some(ref dir) = embf_dir {
        println!("Found EMB directory: {}", dir.display());
        println!("Files will be copied to this directory");
    }

    println!("Watching directory: {}", path.as_ref().display());
    println!("Press 'q' to quit, 't' to select target directory, 'u' to unmount USB volume");

    enable_raw_mode().unwrap();

    const POLL_DURATION: Duration = Duration::from_millis(100);

    'main: loop {
        // Check both keyboard and file events in each iteration
        while let Ok(event) = event_rx.try_recv() {
            match event {
                WatcherEvent::File(Ok(event)) => {
                    if let notify::EventKind::Create(_) = event.kind {
                        for path in event.paths {
                            if let Err(e) = handle_file_creation(&path, &inkscape_info, &embf_dir) {
                                eprintln!("Error handling file creation: {}", e);
                            }
                        }
                    }
                }
                WatcherEvent::File(Err(e)) => println!("Error receiving file event: {}", e),
            }
        }

        // Check for keyboard input
        if event::poll(POLL_DURATION).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match handle_key_event(key, &mut embf_dir) {
                    Ok(true) => break 'main, // Exit requested
                    Ok(false) => (),         // Continue watching
                    Err(e) => {
                        eprintln!("Error handling key event: {}", e);
                        break 'main;
                    }
                }
            }
        }
    }

    disable_raw_mode().unwrap();
}

fn main() {
    let args = Args::parse();

    // Set up signal handlers
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        disable_raw_mode().unwrap();
        println!("Stopping file watcher signal...");
        disable_raw_mode().unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    let inkscape_info = match inkscape::find_inkscape() {
        Some(info) => info,
        None => {
            println!("Inkscape not found. Please download and install from https://inkscape.org/release/1.4/mac-os-x/");
            return;
        }
    };

    let watch_dir = args.dir.unwrap_or_else(|| {
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

    watch_directory(watch_dir, rx, inkscape_info, embf_dir);
    println!("File watcher stopped.");
}
