mod inkscape;
mod usb_drive;
mod utils;

// External crate imports
use clap::Parser;
use ctrlc;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use termion::async_stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

// Standard library imports
use std::error::Error;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

// Internal crate imports
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
    write!(stdout, "{}", termion::cursor::Show).unwrap();
    stdout.flush().unwrap();

    println!("Please enter the path to the target directory:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let path = PathBuf::from(input.trim());

    // Switch back to raw mode
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();

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
    if path.extension() != Some(OsStr::new("dst")) {
        return Ok(());
    }

    println!("New DST file detected: {}", path.display());
    let output_path = sanitize_filename(path);

    let output = Command::new(&inkscape_info.path)
        .arg(path)
        .arg("--export-filename")
        .arg(&output_path)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("extension not found") || error.contains("unknown extension") {
            println!("ink/stitch extension not installed or not working properly. Please download and install from https://inkstitch.org/docs/install/");
        } else {
            println!("Error converting file: {}", error);
        }
        return Ok(());
    }

    println!("Converted to JEF: {}", output_path.display());

    if let Some(ref embf_dir) = embf_dir {
        let dest = embf_dir.join(output_path.file_name().unwrap());
        std::fs::copy(&output_path, &dest)?;
        println!("Copied to EMB directory: {}", dest.display());
    }

    Ok(())
}

fn handle_key_event(
    key: Key,
    stdout: &mut io::Stdout,
    embf_dir: &mut Option<PathBuf>,
) -> Result<bool, io::Error> {
    match key {
        Key::Char('q') | Key::Ctrl('c') | Key::Ctrl('z') => {
            write!(stdout, "{}", termion::cursor::Show)?;
            println!("\nStopping file watcher...");
            Ok(true) // Signal to stop watching
        }
        Key::Char('t') => {
            write!(stdout, "{}", termion::cursor::Show)?;
            stdout.flush()?;

            if let Some(new_dir) = select_copy_target_directory() {
                *embf_dir = Some(new_dir);
                println!("New target directory set.");
            }

            let mut stdout = io::stdout().into_raw_mode()?;
            write!(stdout, "{}", termion::cursor::Hide)?;
            Ok(false)
        }
        Key::Char('u') => {
            unmount_usb_volume();
            Ok(false)
        }
        _ => Ok(false),
    }
}

// Add a new enum to handle both types of events
#[derive(Debug)]
enum WatcherEvent {
    File(notify::Result<Event>),
    Keyboard(Key),
}

// Update the watch_directory signature and implementation
fn watch_directory(
    path: impl AsRef<Path>,
    event_rx: Receiver<WatcherEvent>,
    inkscape_info: InkscapeInfo,
    mut embf_dir: Option<PathBuf>,
    running: Arc<AtomicBool>,
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

    let mut stdout = match io::stdout().into_raw_mode() {
        Ok(stdout) => stdout,
        Err(e) => {
            eprintln!("Error setting up terminal: {}", e);
            return;
        }
    };

    if let Err(e) = write!(stdout, "{}", termion::cursor::Hide) {
        eprintln!("Error hiding cursor: {}", e);
        return;
    }

    const SLEEP_DURATION: Duration = Duration::from_millis(100);

    while running.load(Ordering::SeqCst) {
        match event_rx.recv_timeout(SLEEP_DURATION) {
            Ok(WatcherEvent::Keyboard(key)) => {
                match handle_key_event(key, &mut stdout, &mut embf_dir) {
                    Ok(true) => return, // Exit requested
                    Ok(false) => (),    // Continue watching
                    Err(e) => {
                        eprintln!("Error handling key event: {}", e);
                        return;
                    }
                }
            }
            Ok(WatcherEvent::File(Ok(event))) => match event.kind {
                notify::EventKind::Create(_) => {
                    for path in event.paths {
                        if let Err(e) = handle_file_creation(&path, &inkscape_info, &embf_dir) {
                            eprintln!("Error handling file creation: {}", e);
                        }
                    }
                }
                _ => (),
            },
            Ok(WatcherEvent::File(Err(e))) => println!("Error receiving file event: {}", e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
            Err(e) => {
                eprintln!("Error receiving event: {}", e);
                return;
            }
        }
    }

    // Cleanup when exiting
    write!(stdout, "{}", termion::cursor::Show).unwrap_or_default();
    println!("\nStopping file watcher...");
}

fn main() {
    let args = Args::parse();

    // Set up signal handlers with terminal cleanup early
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let mut stdout_cleanup = io::stdout();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        // Restore terminal on Ctrl-C
        write!(stdout_cleanup, "{}", termion::cursor::Show).unwrap_or_default();
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
    let kbd_tx = fs_tx.clone();

    // Create watcher with modified event sending
    let mut watcher = match RecommendedWatcher::new(
        move |res| {
            eprintln!("event: {:?}", res);
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
        Ok(_) => eprintln!("Successfully set up directory watching"),
        Err(e) => {
            eprintln!("Failed to watch directory: {:?}", e);
            return;
        }
    };

    // Spawn keyboard input thread
    std::thread::spawn(move || {
        let stdin = async_stdin();
        let mut keys = stdin.keys();
        loop {
            if let Some(Ok(key)) = keys.next() {
                if let Err(e) = kbd_tx.send(WatcherEvent::Keyboard(key)) {
                    eprintln!("Error sending keyboard event: {:?}", e);
                    break;
                }
            }
            // Add a small sleep to prevent busy-waiting
            std::thread::sleep(Duration::from_millis(50));
        }
    });

    let embf_dir = find_embf_directory();

    // Store watcher in a variable to keep it alive
    let _watcher_guard = watcher;

    watch_directory(watch_dir, rx, inkscape_info, embf_dir, running);
    println!("File watcher stopped.");

    write!(io::stdout(), "{}", termion::cursor::Show).unwrap_or_default();
}
