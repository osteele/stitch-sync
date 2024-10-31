use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use crate::services::inkscape::{self, Inkscape};
use crate::types::format::FileFormat;
use crate::utils;
use crate::utils::color::red;
use crate::utils::sanitize_filename;

fn should_convert_file(path: &Path, inkscape_info: &Inkscape, output_format: &str) -> bool {
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
    inkscape_info: &Inkscape,
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
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let dot_interval = Duration::from_secs(1);
    let poll_interval = Duration::from_millis(50);
    utils::wait_with_progress(&mut child, dot_interval, poll_interval)?;

    let output = child.wait_with_output()?;

    if !output.stdout.is_empty() {
        println!(
            "\nInkscape output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    if !output.stderr.is_empty() {
        println!(
            "\nInkscape error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let error = String::from_utf8_lossy(&output.stderr);
    if error.contains("extension not found")
        || error.contains("unknown extension")
        || error.contains("Could not detect file format")
    {
        let msg = format!(
            "ink/stitch extension not installed or not working properly. Please download and install from {}",
            inkscape::INKSTITCH_INSTALL_URL
        );
        return Err(msg.into());
    } else if !output.status.success() {
        println!("{}", red(&format!("Error converting file: {}", error)));
        return Err("Inkscape conversion failed".into());
    }

    let elapsed = start.elapsed();
    println!(
        "  Converted to {} format: {} ({:.2}s elapsed time)",
        output_format,
        output_path.display(),
        elapsed.as_secs_f32()
    );

    Ok(output_path)
}

pub fn handle_file_creation(
    path: &Path,
    inkscape_info: &Inkscape,
    embf_dir: &Option<PathBuf>,
    accepted_formats: &[String],
    preferred_format: &str,
) -> Result<(), Box<dyn Error>> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if FileFormat::find_by_extension(&extension).is_some()
        || accepted_formats.iter().any(|fmt| fmt == &extension)
        || inkscape_info
            .supported_read_formats
            .contains(&extension.as_str())
        || inkscape_info
            .supported_write_formats
            .contains(&extension.as_str())
    {
        println!("New file detected: {}", path.display());
    }
    // Go ahead and proceed with the rest of the logic even if it's not a file
    // we recognize, since our list of extensions is not exhaustive

    // If the file is in an accepted format, just copy it
    if accepted_formats.iter().any(|fmt| fmt == &extension) {
        if let Some(ref embf_dir) = embf_dir {
            println!("  Copying {} to target directory...", path.display());
            let dest = embf_dir.join(path.file_name().unwrap());
            std::fs::copy(path, &dest)?;
            println!("  Copied to: {}", dest.display());
        } else {
            // println!("  Already in the correct format, skipping conversion");
            // println!("  No copy target directory specified, skipping copy");
        }
        return Ok(());
    }

    // Check if we can convert the file
    if !should_convert_file(path, inkscape_info, preferred_format) {
        return Ok(());
    }

    // Convert the file to preferred format
    match convert_file(path, inkscape_info, preferred_format) {
        Ok(output_path) => {
            if let Some(ref embf_dir) = embf_dir {
                let dest = embf_dir.join(output_path.file_name().unwrap());
                std::fs::copy(&output_path, &dest)?;
                println!("  Copied to target directory: {}", dest.display());
            } else {
                // println!("  No copy target directory specified, skipping copy");
            }
        }
        Err(e) => {
            println!("{}", red(&format!("Error converting file: {}", e)));
        }
    }

    Ok(())
}
