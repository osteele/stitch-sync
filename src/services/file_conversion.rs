use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::services::Inkscape;
use crate::services::UsbDrive;
use crate::utils::sanitize_filename;

fn convert_file(
    input_path: &Path,
    inkscape: &Inkscape,
    output_format: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut stdout = io::stdout();
    print!(
        "Converting {} to {} using Inkscape...",
        input_path.display(),
        output_format
    );
    stdout.flush()?;

    let output_path = sanitize_filename(input_path).with_extension(output_format);

    let start = Instant::now();
    inkscape.convert_file(input_path, &output_path)?;
    let elapsed = start.elapsed();

    println!("done ({:.2}s elapsed time)", elapsed.as_secs_f32());

    Ok(output_path)
}

fn copy_file_to_usb_drives(source_path: &Path, usb_rel_path: &str) -> Result<(), Box<dyn Error>> {
    let drives = UsbDrive::find_usb_drives();
    let target_paths = drives
        .iter()
        .map(|drive| drive.mount_point.join(usb_rel_path))
        .filter(|path| path.exists())
        .collect::<Vec<PathBuf>>();

    match (drives.len(), target_paths.len()) {
        (0, _) => println!("New file detected: {}", source_path.display()),
        (_, 0) => println!(
            "New file {} will not be copied. USB drive{} found, but none contains target path {}.",
            if drives.len() > 1 { "s" } else { "" },
            usb_rel_path,
            source_path.display()
        ),
        (_, 1) => (),
        (_, _) => println!(
            "Multiple USB drives found; selecting {}...",
            target_paths.first().unwrap().display()
        ),
    }
    if let Some(target_dir) = target_paths.first() {
        let filename = source_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid source path"))?;
        let dest = target_dir.join(sanitize_filename(Path::new(filename)));
        std::fs::copy(source_path, &dest)?;
        println!("Copied {} to {}", source_path.display(), dest.display());
    }
    Ok(())
}

pub fn handle_file_detection(
    path: &Path,
    inkscape: &Inkscape,
    usb_target_path: &Option<&str>,
    accepted_formats: &[&str],
    preferred_format: &str,
) -> Result<(), Box<dyn Error>> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if accepted_formats.contains(&extension.as_str()) {
        if let Some(usb_rel_path) = usb_target_path {
            copy_file_to_usb_drives(path, usb_rel_path)?;
        }
    } else if inkscape
        .supported_read_formats
        .contains(&extension.as_str())
        && inkscape.supported_write_formats.contains(&preferred_format)
    {
        convert_file(path, inkscape, preferred_format)?;
    }
    Ok(())
}
