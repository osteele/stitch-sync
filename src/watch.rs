use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use notify::Event as NotifyEvent;
use scopeguard::defer;

use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::usb_drive::unmount_usb_volume;
use crate::{file_conversion::handle_file_creation, inkscape::InkscapeInfo};

#[derive(Debug)]
pub enum WatcherEvent {
    File(notify::Result<NotifyEvent>),
}

pub fn watch_directory(
    path: impl AsRef<Path>,
    event_rx: Receiver<WatcherEvent>,
    inkscape_info: InkscapeInfo,
    embf_dir: Option<PathBuf>,
    accepted_formats: Vec<String>,
    preferred_format: String,
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
                                &embf_dir,
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
