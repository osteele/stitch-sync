use regex::Regex;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    Win32::Storage::FileSystem::{
        CreateFileW, GetDriveTypeW, FILE_FLAG_SEQUENTIAL_SCAN, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    },
    Win32::System::Ioctl::IOCTL_STORAGE_EJECT_MEDIA,
    Win32::System::IO::DeviceIoControl,
};

#[cfg(target_os = "linux")]
use libudev::Enumerator;

pub struct UsbDrive {
    pub mount_point: PathBuf,
    pub name: String,
}

impl UsbDrive {
    #[cfg(target_os = "windows")]
    fn is_usb_drive(path: &Path) -> bool {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let path_str = path.to_str().unwrap_or("");
        if path_str.len() < 2 {
            return false;
        }

        let mut wide: Vec<u16> = OsStr::new(&format!("{}\\", path_str))
            .encode_wide()
            .collect();
        wide.push(0);

        unsafe { GetDriveTypeW(PCWSTR::from_raw(wide.as_ptr())) == 2 }
    }

    #[cfg(target_os = "linux")]
    fn is_usb_drive(path: &Path) -> bool {
        let udev = match libudev::Context::new() {
            Ok(udev) => udev,
            Err(_) => return false,
        };

        let mut enumerator = match Enumerator::new(&udev) {
            Ok(enum_) => enum_,
            Err(_) => return false,
        };

        enumerator.match_subsystem("usb").ok();

        let device_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return false,
        };

        if let Ok(devices) = enumerator.scan_devices() {
            for device in devices {
                if let Some(devnode) = device.devnode() {
                    if devnode == device_path {
                        if let Some(parent) = device.parent() {
                            return parent.subsystem().map_or(false, |s| s == "usb");
                        }
                    }
                }
            }
        }
        false
    }

    #[cfg(target_os = "macos")]
    fn is_usb_drive(path: &Path) -> bool {
        use std::process::Command;

        // Get the device identifier for the given path
        let output = match Command::new("diskutil").arg("info").arg(path).output() {
            Ok(output) => output,
            Err(_) => return false,
        };

        let info = String::from_utf8_lossy(&output.stdout);

        let removable_re = Regex::new(r"^\s*Removable Media:\s+(Yes|Removable)\s*$").unwrap();
        let protocol_re = Regex::new(r"^\s*Protocol:\s+USB\s*$").unwrap();

        info.lines().any(|line| removable_re.is_match(line))
            && info.lines().any(|line| protocol_re.is_match(line))
    }

    pub fn list() -> Vec<UsbDrive> {
        #[cfg(target_os = "macos")]
        {
            let volumes = Path::new("/Volumes");
            if !volumes.exists() {
                return vec![];
            }

            std::fs::read_dir(volumes)
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if Self::is_usb_drive(&path) {
                        Some(UsbDrive {
                            name: entry.file_name().to_string_lossy().into_owned(),
                            mount_point: path,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }

        #[cfg(target_os = "windows")]
        {
            (b'A'..=b'Z')
                .filter_map(|drive_letter| {
                    let drive = PathBuf::from(format!("{}:", drive_letter as char));
                    if drive.exists() && Self::is_usb_drive(&drive) {
                        Some(UsbDrive {
                            name: format!("Drive ({}:)", drive_letter as char),
                            mount_point: drive,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }

        #[cfg(target_os = "linux")]
        {
            let media = Path::new("/media");
            if let Some(username) = std::env::var_os("USER") {
                let user_media = media.join(username);
                if user_media.exists() {
                    return std::fs::read_dir(user_media)
                        .into_iter()
                        .flatten()
                        .filter_map(|entry| {
                            let entry = entry.ok()?;
                            let path = entry.path();
                            if Self::is_usb_drive(&path) {
                                Some(UsbDrive {
                                    name: entry.file_name().to_string_lossy().into_owned(),
                                    mount_point: path,
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            }
            vec![]
        }
    }

    pub fn unmount(&self) {
        #[cfg(target_os = "macos")]
        {
            let result = Command::new("diskutil")
                .arg("eject")
                .arg(&self.mount_point)
                .output();

            match result {
                Ok(output) if output.status.success() => {
                    println!("Successfully ejected drive: {}", self.name);
                }
                Ok(output) => {
                    println!(
                        "Error ejecting drive: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(e) => println!("Error running diskutil: {}", e),
            }
        }

        #[cfg(target_os = "linux")]
        {
            let result = Command::new("umount").arg(&self.mount_point).output();

            match result {
                Ok(output) if output.status.success() => {
                    let _ = Command::new("udisksctl")
                        .arg("power-off")
                        .arg("-b")
                        .arg(&self.mount_point)
                        .output();
                    println!("Successfully ejected drive: {}", self.name);
                }
                Ok(output) => {
                    println!(
                        "Error ejecting drive: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(e) => println!("Error running umount: {}", e),
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use std::ptr;

            unsafe {
                let device_path = format!(
                    "\\\\.\\{}:",
                    self.mount_point
                        .to_str()
                        .unwrap_or("")
                        .chars()
                        .next()
                        .unwrap()
                );
                let wide_path: Vec<u16> = OsStr::new(&device_path)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                let handle_result = CreateFileW(
                    PCWSTR::from_raw(wide_path.as_ptr()),
                    0x80000000 | 0x40000000, // GENERIC_READ | GENERIC_WRITE
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    Some(ptr::null()),
                    OPEN_EXISTING,
                    FILE_FLAG_SEQUENTIAL_SCAN,
                    HANDLE(0),
                );

                match handle_result {
                    Ok(handle) => {
                        if handle == INVALID_HANDLE_VALUE {
                            println!("Error opening drive handle");
                            return;
                        }

                        // Try to eject the media
                        let mut bytes_returned: u32 = 0;
                        let result = DeviceIoControl(
                            handle,
                            IOCTL_STORAGE_EJECT_MEDIA,
                            None,
                            0,
                            None,
                            0,
                            Some(&mut bytes_returned),
                            None,
                        );

                        // Close handle before checking result
                        let _ = CloseHandle(handle);

                        if result.as_bool() {
                            println!("Successfully ejected drive: {}", self.name);
                        } else {
                            println!("Error ejecting drive");
                        }
                    }
                    Err(_) => {
                        println!("Failed to open drive handle");
                    }
                }
            }
        }
    }
}

pub fn find_usb_containing_path(path: &str) -> Option<PathBuf> {
    UsbDrive::list()
        .into_iter()
        .map(|drive| drive.mount_point)
        .find(|mount_point| mount_point.join(path).is_dir())
        .map(|mount_point| mount_point.join(path))
}

pub fn unmount_usb_volume() {
    let drives = UsbDrive::list();

    match drives.len() {
        0 => {
            println!("No USB drives found.");
        }
        1 => {
            println!("Ejecting USB drive: {}", drives[0].name);
            drives[0].unmount();
        }
        _ => {
            println!("Multiple USB drives found. Please choose one (or 'q' to quit):");
            for (i, drive) in drives.iter().enumerate() {
                println!("{}. {}", i + 1, drive.name);
            }

            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            let input = input.trim();

            if input.eq_ignore_ascii_case("q") {
                return;
            }

            if let Ok(choice) = input.parse::<usize>() {
                if choice > 0 && choice <= drives.len() {
                    drives[choice - 1].unmount();
                } else {
                    println!("Invalid selection.");
                }
            } else {
                println!("Invalid input.");
            }
        }
    }
}
