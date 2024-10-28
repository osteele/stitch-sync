use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_embf_directory() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let volumes = Path::new("/Volumes");
        if !volumes.exists() {
            return None;
        }

        std::fs::read_dir(volumes).ok()?.find_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path().join("EMB").join("Embf");
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
    }

    #[cfg(target_os = "windows")]
    {
        // Get all available drives
        for drive_letter in b'A'..=b'Z' {
            let drive = PathBuf::from(format!("{}:\\", drive_letter as char));
            if drive.exists() {
                let embf_path = drive.join("EMB").join("Embf");
                if embf_path.is_dir() {
                    return Some(embf_path);
                }
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    {
        let media = Path::new("/media");
        if let Some(username) = std::env::var_os("USER") {
            let user_media = media.join(username);
            if user_media.exists() {
                return std::fs::read_dir(user_media).ok()?.find_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path().join("EMB").join("Embf");
                    if path.is_dir() {
                        Some(path)
                    } else {
                        None
                    }
                });
            }
        }
        None
    }
}

pub fn unmount_usb_volume() {
    #[cfg(target_os = "macos")]
    {
        println!("Please enter the name of the USB volume to unmount:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let volume_name = input.trim();

        let result = Command::new("diskutil")
            .arg("unmount")
            .arg(format!("/Volumes/{}", volume_name))
            .output();

        match result {
            Ok(output) if output.status.success() => {
                println!("Successfully unmounted volume: {}", volume_name);
            }
            Ok(output) => {
                println!(
                    "Error unmounting volume: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => println!("Error running diskutil: {}", e),
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("Please enter the mount point of the USB volume to unmount:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let mount_point = input.trim();

        let result = Command::new("umount").arg(mount_point).output();

        match result {
            Ok(output) if output.status.success() => {
                println!("Successfully unmounted volume: {}", mount_point);
            }
            Ok(output) => {
                println!(
                    "Error unmounting volume: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => println!("Error running umount: {}", e),
        }
    }

    #[cfg(target_os = "windows")]
    {
        println!("Unmounting USB volumes is not supported on Windows in this example.");
    }
}
