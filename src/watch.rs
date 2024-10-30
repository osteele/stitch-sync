use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ctrlc;
use notify::Event as NotifyEvent;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use scopeguard::defer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;

use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::usb_drive::unmount_usb_volume;
use crate::{file_conversion::handle_file_creation, inkscape::InkscapeInfo};
use crate::{inkscape, machines::MachineInfo};

#[derive(Debug)]
pub enum WatcherEvent {
    File(notify::Result<NotifyEvent>),
}

pub fn watch(
    watch_dir: Option<PathBuf>,
    copy_target_dir: Option<PathBuf>,
    output_format: Option<String>,
    machine: &Option<MachineInfo>,
) {
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

    // Determine accepted formats and preferred format
    let (accepted_formats, preferred_format) = match &machine {
        Some(machine) => {
            let formats: Vec<String> = machine
                .formats
                .iter()
                .map(|f| f.extension.to_string())
                .collect();
            let preferred = output_format
                .or_else(|| formats.first().map(|s| s.to_string()))
                .unwrap_or_else(|| "dst".to_string());
            (formats, preferred)
        }
        None => {
            let preferred = output_format.unwrap_or_else(|| "dst".to_string());
            (vec![preferred.clone()], preferred)
        }
    };

    let watch_dir = watch_dir.unwrap_or_else(|| {
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

    watch_directory(
        watch_dir,
        rx,
        inkscape_info,
        copy_target_dir,
        accepted_formats,
        preferred_format,
    );
    println!("File watcher stopped.");
}

pub fn watch_directory(
    path: impl AsRef<Path>,
    event_rx: Receiver<WatcherEvent>,
    inkscape_info: InkscapeInfo,
    copy_target_dir: Option<PathBuf>,
    accepted_formats: Vec<String>,
    preferred_format: String,
) {
    let warn_inkstitch = false;

    if warn_inkstitch && !inkscape_info.has_inkstitch {
        println!("Warning: ink/stitch extension not found. Please install from https://inkstitch.org/docs/install/");
    }

    if let Some(ref dir) = copy_target_dir {
        println!("Found EMB directory: {}", dir.display());
        println!("Files will be copied to this directory");
    }

    println!("Watching directory: {}", path.as_ref().display());
    println!("Press 'q' to quit, 'u' to unmount USB volume");

    enable_raw_mode().unwrap();
    defer! {
        disable_raw_mode().unwrap();
    }

    const POLL_DURATION: Duration = Duration::from_millis(100);

    'main: loop {
        // Check both keyboard and file events in each iteration
        while let Ok(event) = event_rx.try_recv() {
            disable_raw_mode().unwrap();
            match event {
                WatcherEvent::File(Ok(event)) => {
                    if let notify::EventKind::Create(_) = event.kind {
                        for path in event.paths {
                            if let Err(e) = handle_file_creation(
                                &path,
                                &inkscape_info,
                                &copy_target_dir,
                                &accepted_formats,
                                &preferred_format,
                            ) {
                                eprintln!("Error handling file creation: {}", e);
                            }
                        }
                    }
                }
                WatcherEvent::File(Err(e)) => println!("Error receiving file event: {}", e),
            }
            enable_raw_mode().unwrap();
        }

        // Check for keyboard input
        if event::poll(POLL_DURATION).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                disable_raw_mode().unwrap();
                match handle_key_event(key) {
                    Ok(true) => break 'main, // Exit requested
                    Ok(false) => (),         // Continue watching
                    Err(e) => {
                        eprintln!("Error handling key event: {}", e);
                        break 'main;
                    }
                }
            }
            enable_raw_mode().unwrap();
        }
    }
}

// Returns true if the program should exit
fn handle_key_event(key: KeyEvent) -> Result<bool, io::Error> {
    match (key.code, key.modifiers.contains(KeyModifiers::CONTROL)) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), true) => Ok(true),
        (KeyCode::Char('u'), _) => {
            unmount_usb_volume();
            Ok(false)
        }
        _ => Ok(false),
    }
}
