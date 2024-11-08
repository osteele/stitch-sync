use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use notify::Event as NotifyEvent;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use scopeguard::defer;

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::SystemTime;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread::sleep,
    time::Duration,
};

use crate::services::usb_drive::unmount_usb_volume;
use crate::services::{
    file_conversion::handle_file_detection,
    inkscape::{self, Inkscape},
};
use crate::utils::WATCH_POLL_INTERVAL;

// Option 1: Scanning folder animation
const CURSOR_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const FRAME_DURATION: Duration = Duration::from_millis(200);

#[derive(Debug)]
pub enum WatcherEvent {
    File(notify::Result<NotifyEvent>),
}

#[derive(Debug, Clone, PartialEq)]
struct FileMetadata {
    modified: SystemTime,
    size: u64,
}

struct FileCache {
    cache: HashMap<PathBuf, FileMetadata>,
}

impl FileCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn filter_new_files<'a>(
        &'a mut self,
        paths: &'a [PathBuf],
    ) -> impl Iterator<Item = &'a PathBuf> {
        paths.iter().filter(|&path| {
            if let Ok(metadata) = std::fs::metadata(path) {
                let current_metadata = FileMetadata {
                    modified: metadata.modified().unwrap_or(SystemTime::now()),
                    size: metadata.len(),
                };

                match self.cache.get(path) {
                    Some(cached_metadata) if cached_metadata == &current_metadata => false,
                    _ => {
                        self.cache.insert(path.clone(), current_metadata);
                        true
                    }
                }
            } else {
                false
            }
        })
    }
}

pub fn watch(
    watch_dir: &PathBuf,
    usb_target_path: &Option<&str>,
    accepted_formats: &[&str],
    preferred_format: &str,
) {
    // Set up signal handlers
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let inkscape = match Inkscape::find_app() {
        Some(info) => info,
        None => {
            println!(
                "Inkscape not found. Please download and install from {}",
                inkscape::INKSCAPE_DOWNLOAD_URL
            );
            return;
        }
    };

    if !watch_dir.exists() {
        println!("Directory does not exist: {}", watch_dir.display());
        return;
    }

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
    match watcher.watch(watch_dir, RecursiveMode::NonRecursive) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to watch directory: {:?}", e);
            return;
        }
    };

    watch_directory(
        watch_dir,
        rx,
        inkscape,
        usb_target_path,
        accepted_formats,
        preferred_format,
    );
    println!("File watcher stopped.");
}

pub fn watch_directory(
    _path: impl AsRef<Path>,
    event_rx: Receiver<WatcherEvent>,
    inkscape: Inkscape,
    usb_target_path: &Option<&str>,
    accepted_formats: &[&str],
    preferred_format: &str,
) {
    let mut file_cache = FileCache::new();
    let mut frame_index = 0;
    let mut last_frame = SystemTime::now();

    enable_raw_mode().unwrap();
    defer! {
        disable_raw_mode().unwrap();
        // Clear the cursor line when exiting
        print!("\r\x1B[K");
        let _ = io::stdout().flush();
    }

    'main: loop {
        // Update spinner animation
        if last_frame.elapsed().unwrap_or_default() >= FRAME_DURATION {
            print!(
                "\r👀 Watching for new stitch files... {}",
                CURSOR_FRAMES[frame_index]
            );
            let _ = io::stdout().flush();
            frame_index = (frame_index + 1) % CURSOR_FRAMES.len();
            last_frame = SystemTime::now();
        }

        // Check both keyboard and file events in each iteration
        while let Ok(event) = event_rx.try_recv() {
            disable_raw_mode().unwrap();
            // Clear the cursor line before processing file
            print!("\r\x1B[K");
            let _ = io::stdout().flush();

            match event {
                WatcherEvent::File(Ok(event)) => {
                    let paths = match event.kind {
                        notify::EventKind::Create(_) => event.paths,
                        notify::EventKind::Modify(_) => {
                            sleep(Duration::from_millis(150)); // give the file time to settle
                            event.paths
                        }
                        _ => vec![],
                    };

                    // Use the new filter_new_files method
                    for path in file_cache.filter_new_files(&paths) {
                        if let Err(e) = handle_file_detection(
                            path,
                            &inkscape,
                            usb_target_path,
                            accepted_formats,
                            preferred_format,
                        ) {
                            eprintln!("Error handling file creation: {}", e);
                        }
                    }
                }
                WatcherEvent::File(Err(e)) => println!("Error receiving file event: {}", e),
            }
            enable_raw_mode().unwrap();
        }

        // Check for keyboard input
        if event::poll(WATCH_POLL_INTERVAL).unwrap() {
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
