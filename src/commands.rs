use anyhow::Result;
use std::path::PathBuf;

use crate::config::defaults::DEFAULT_FORMAT;
use crate::config::ConfigManager;
use crate::services;
use crate::services::find_usb_containing_path;
use crate::services::inkscape;
use crate::services::Inkscape;
use crate::types::Machine;
use crate::types::MACHINES;
use crate::utils::color::red;

pub fn list_machines_command(format: Option<String>, verbose: bool) -> Result<()> {
    let machines = if let Some(format) = format {
        MACHINES
            .iter()
            .filter(|m| m.formats.contains(&format.to_lowercase()))
            .collect::<Vec<_>>()
    } else {
        MACHINES.iter().collect()
    };

    for machine in machines {
        if verbose {
            println!("{}", machine.name);
            if !machine.synonyms.is_empty() {
                println!("  Synonyms: {}", machine.synonyms.join(", "));
            }
            if let Some(notes) = &machine.notes {
                println!("  Note: {}", notes);
            }
            if let Some(design_size) = &machine.design_size {
                println!("  Design size: {}", design_size);
            }
            if let Some(usb_path) = &machine.usb_path {
                println!("  USB path: {}", usb_path);
            }
        } else {
            println!("{} ({})", machine.name, machine.formats.join(", "));
        }
    }
    Ok(())
}

pub fn watch_command(
    watch_dir: Option<PathBuf>,
    output_format: Option<String>,
    machine_name: Option<String>,
) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    let inkscape = Inkscape::find_app();
    if inkscape.is_none() {
        println!(
            "Inkscape not found. Please download and install from {}",
            inkscape::INKSCAPE_DOWNLOAD_URL
        );
        println!("Opening download page in your browser...");
        services::open_browser(inkscape::INKSCAPE_DOWNLOAD_URL);
        return Ok(());
    }
    if !inkscape.as_ref().unwrap().has_inkstitch {
        println!(
            "{}",
            red(&format!(
                "Warning: ink/stitch extension not found. Please install from {}",
                inkscape::INKSTITCH_INSTALL_URL
            ))
        );
    }

    let watch_dir = watch_dir.or(config.watch_dir).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join("Downloads")
    });

    let machine_name = machine_name.or(config.machine);
    let machine = machine_name
        .as_ref()
        .and_then(|m| Machine::interactive_find_by_name(m));
    if machine_name.is_some() && machine.is_none() {
        println!("Machine '{}' not found", machine_name.unwrap());
        return Ok(());
    }

    let copy_target_path = machine
        .as_ref()
        .and_then(|m| m.usb_path.as_deref())
        .unwrap_or("");
    let copy_target_dir = find_usb_containing_path(copy_target_path);

    // Determine accepted formats and preferred format
    let (accepted_formats, preferred_format) = match &machine {
        Some(machine) => {
            let formats = machine.formats.clone();
            let preferred = output_format
                .or_else(|| formats.first().map(|s| s.to_string()))
                .unwrap_or_else(|| DEFAULT_FORMAT.to_string())
                .to_lowercase();
            (formats, preferred)
        }
        None => {
            let preferred = output_format.unwrap_or_else(|| DEFAULT_FORMAT.to_string());
            (vec![preferred.clone()], preferred)
        }
    };

    // Convert preferred format to 'jef' if it ends with 'jef+'
    let preferred_format = if preferred_format == "jef+"
        && !inkscape
            .as_ref()
            .unwrap()
            .supported_write_formats
            .contains(&preferred_format.as_str())
    {
        "jef".to_string()
    } else {
        preferred_format
    };

    println!("Watching directory: {}", watch_dir.display());
    if let Some(machine) = machine {
        println!("Machine: {}", machine.name);
    }
    // let mut watch_formats = Vec::new();
    // watch_formats.extend(
    //     inkscape
    //         .unwrap()
    //         .supported_read_formats
    //         .iter()
    //         .map(|f| f.to_string()),
    // );
    // watch_formats.extend(accepted_formats.clone());
    // watch_formats.sort();
    // watch_formats.dedup();
    // println!("Watching for formats: {}", watch_formats.join(", "));

    services::watch_dir(
        watch_dir,
        copy_target_dir,
        accepted_formats,
        preferred_format,
    );
    Ok(())
}
