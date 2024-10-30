use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use notify::Event as NotifyEvent;
use scopeguard::defer;

use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::time::Instant;

use crate::inkscape::InkscapeInfo;
use crate::usb_drive::unmount_usb_volume;
use crate::utils;
use crate::utils::sanitize_filename;

#[derive(Debug)]
pub enum WatcherEvent {
    File(notify::Result<NotifyEvent>),
}

pub fn watch_directory(
    path: impl AsRef<Path>,
    event_rx: Receiver<WatcherEvent>,
    inkscape_info: InkscapeInfo,
    embf_dir: Option<PathBuf>,
    output_format: String,
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
                                &output_format,
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

/// Handles a file creation event, converting DST to JEF and copying to target directory
fn handle_file_creation(
    path: &Path,
    inkscape_info: &InkscapeInfo,
    embf_dir: &Option<PathBuf>,
    output_format: &str,
) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    // If the file is already in the target format, just copy it
    if extension == output_format {
        if let Some(ref embf_dir) = embf_dir {
            println!("Copying {} to EMB directory...", path.display());
            let dest = embf_dir.join(path.file_name().unwrap());
            std::fs::copy(path, &dest)?;
            println!("Copied to: {}", dest.display());
        }
        return Ok(());
    }

    // Check if input format is supported
    if !inkscape_info
        .supported_read_formats
        .contains(&extension.as_str())
    {
        return Ok(());
    }

    // Check if output format is supported
    let image_formats = ["png", "jpg", "jpeg", "tiff", "bmp", "gif", "webp"];
    if !inkscape_info
        .supported_write_formats
        .contains(&output_format)
        && !image_formats.contains(&output_format)
    {
        println!(
            "Warning: Output format '{}' is not supported by Inkscape",
            output_format
        );
        return Ok(());
    }

    println!("New file detected: {}", path.display());
    print!(
        "Converting {} to {} using Inkscape...",
        path.display(),
        output_format
    );
    stdout.flush()?;

    let mut output_path = sanitize_filename(path);
    output_path.set_extension(output_format);
    let start = Instant::now();

    // Start the command
    let mut child = Command::new(&inkscape_info.path)
        .arg(path)
        .arg("--export-filename")
        .arg(&output_path)
        .spawn()?;

    // Print dots while waiting
    let dot_interval = Duration::from_secs(1);
    let poll_interval = Duration::from_millis(50);
    utils::wait_with_progress(&mut child, dot_interval, poll_interval)?;

    // Get the final status
    let status = child.wait()?;
    println!("done");

    if !status.success() {
        // Get error output
        let output = child.wait_with_output()?;
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("extension not found") || error.contains("unknown extension") {
            println!("ink/stitch extension not installed or not working properly. Please download and install from https://inkstitch.org/docs/install/");
        } else {
            println!("Error converting file: {}", error);
        }
        return Ok(());
    }

    let elapsed = start.elapsed();
    println!(
        "Converted to {}: {} ({:.2}s)",
        output_format,
        output_path.display(),
        elapsed.as_secs_f32()
    );

    if let Some(ref embf_dir) = embf_dir {
        let dest = embf_dir.join(output_path.file_name().unwrap());
        std::fs::copy(&output_path, &dest)?;
        println!("Copied to EMB directory: {}", dest.display());
    }

    Ok(())
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
