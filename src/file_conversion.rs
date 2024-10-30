use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use crate::inkscape::InkscapeInfo;
use crate::utils;
use crate::utils::sanitize_filename;

fn should_convert_file(path: &Path, inkscape_info: &InkscapeInfo, output_format: &str) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    // Don't convert if already in target format
    if extension == output_format {
        return false;
    }

    // Check if input format is supported
    if !inkscape_info
        .supported_read_formats
        .contains(&extension.as_str())
    {
        return false;
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
        return false;
    }

    true
}

fn convert_file(
    path: &Path,
    inkscape_info: &InkscapeInfo,
    output_format: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut stdout = io::stdout();
    print!(
        "Converting {} to {} using Inkscape...",
        path.display(),
        output_format
    );
    stdout.flush()?;

    let mut output_path = sanitize_filename(path);
    output_path.set_extension(output_format);
    let start = Instant::now();

    let mut child = Command::new(&inkscape_info.path)
        .arg(path)
        .arg("--export-filename")
        .arg(&output_path)
        .spawn()?;

    let dot_interval = Duration::from_secs(1);
    let poll_interval = Duration::from_millis(50);
    utils::wait_with_progress(&mut child, dot_interval, poll_interval)?;

    let status = child.wait()?;
    println!("done");

    if !status.success() {
        let output = child.wait_with_output()?;
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("extension not found") || error.contains("unknown extension") {
            println!("ink/stitch extension not installed or not working properly. Please download and install from https://inkstitch.org/docs/install/");
        } else {
            println!("Error converting file: {}", error);
        }
        return Err("Conversion failed".into());
    }

    let elapsed = start.elapsed();
    println!(
        "Converted to {}: {} ({:.2}s)",
        output_format,
        output_path.display(),
        elapsed.as_secs_f32()
    );

    Ok(output_path)
}

pub fn handle_file_creation(
    path: &Path,
    inkscape_info: &InkscapeInfo,
    embf_dir: &Option<PathBuf>,
    output_format: &str,
) -> Result<(), Box<dyn Error>> {
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

    if !should_convert_file(path, inkscape_info, output_format) {
        return Ok(());
    }

    println!("New file detected: {}", path.display());

    if let Ok(output_path) = convert_file(path, inkscape_info, output_format) {
        if let Some(ref embf_dir) = embf_dir {
            let dest = embf_dir.join(output_path.file_name().unwrap());
            std::fs::copy(&output_path, &dest)?;
            println!("Copied to EMB directory: {}", dest.display());
        }
    }

    Ok(())
}
