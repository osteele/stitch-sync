use lazy_static::lazy_static;

use std::error::Error;
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use which::which;

use crate::utils::{self, color::red};

pub const INKSCAPE_DOWNLOAD_URL: &str = "https://inkscape.org/en/download/";

#[cfg(target_os = "windows")]
pub const INKSTITCH_INSTALL_URL: &str = "https://inkstitch.org/docs/install-windows/";

#[cfg(target_os = "macos")]
pub const INKSTITCH_INSTALL_URL: &str = "https://inkstitch.org/docs/install-macos/";

#[cfg(target_os = "linux")]
pub const INKSTITCH_INSTALL_URL: &str = "https://inkstitch.org/docs/install-linux/";

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub const INKSTITCH_INSTALL_URL: &str = "https://inkstitch.org/docs/install/";

lazy_static! {
    /// File formats that Ink/Stitch can write/export
    pub static ref SUPPORTED_WRITE_FORMATS: Vec<&'static str> = vec![
        "csv", "dst", "exp", "jef", "pec", "pes", "svg", "txt", "u01", "vp3"
    ];

    /// File formats that Ink/Stitch can read/import
    pub static ref SUPPORTED_READ_FORMATS: Vec<&'static str> = vec![
        "100", "10o", "bro", "dat", "dsb", "dst", "dsz", "emd", "exp", "exy",
        "fxy", "gt", "inb", "jef", "jpx", "ksm", "max", "mit", "new", "pcd",
        "pcm", "pcq", "pcs", "pec", "pes", "phb", "phc", "sew", "shv", "stc",
        "stx", "tap", "tbf", "txt", "u01", "vp3", "xxx", "zxy"
    ];
}

pub struct Inkscape {
    pub path: PathBuf,
    pub has_inkstitch: bool,
    pub supported_read_formats: &'static [&'static str],
    pub supported_write_formats: &'static [&'static str],
}

impl Inkscape {
    pub fn find_app() -> Option<Inkscape> {
        Self::find_path().map(|path| {
            let has_inkstitch = Self::find_inkstitch_extension(&path);
            Inkscape {
                path,
                has_inkstitch,
                supported_read_formats: &SUPPORTED_READ_FORMATS,
                supported_write_formats: &SUPPORTED_WRITE_FORMATS,
            }
        })
    }

    pub fn convert_file(
        &self,
        path: &Path,
        output_path: &PathBuf,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let mut child = Command::new(&self.path)
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
                INKSTITCH_INSTALL_URL
            );
            return Err(msg.into());
        } else if !output.status.success() {
            println!("{}", red(&format!("Error converting file: {}", error)));
            return Err("Inkscape conversion failed".into());
        }

        Ok(output_path.to_path_buf())
    }

    fn find_path() -> Option<PathBuf> {
        // First try the PATH as it works on all platforms
        if let Ok(path) = which("inkscape") {
            return Some(path);
        }

        // Platform-specific locations
        #[cfg(target_os = "macos")]
        {
            let app_path = PathBuf::from("/Applications/Inkscape.app/Contents/MacOS/inkscape");
            if app_path.exists() {
                return Some(app_path);
            }
        }

        #[cfg(target_os = "windows")]
        {
            let program_files = std::env::var("ProgramFiles").ok();
            let program_files_x86 = std::env::var("ProgramFiles(x86)").ok();

            let possible_paths = vec![
                program_files.as_ref().map(|pf| {
                    PathBuf::from(pf)
                        .join("Inkscape")
                        .join("bin")
                        .join("inkscape.exe")
                }),
                program_files_x86.as_ref().map(|pf| {
                    PathBuf::from(pf)
                        .join("Inkscape")
                        .join("bin")
                        .join("inkscape.exe")
                }),
            ];

            for path in possible_paths.into_iter().flatten() {
                if path.exists() {
                    return Some(path);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let linux_paths = vec![
                "/usr/bin/inkscape",
                "/usr/local/bin/inkscape",
                "/opt/inkscape/bin/inkscape",
            ];

            for path in linux_paths {
                let path = PathBuf::from(path);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    fn find_inkstitch_extension(inkscape_path: &Path) -> bool {
        #[cfg(target_os = "macos")]
        {
            // Check in user's extensions directory
            if let Some(home) = dirs::home_dir() {
                let user_ext = home
                    .join("Library")
                    .join("Application Support")
                    .join("org.inkscape.Inkscape")
                    .join("config")
                    .join("inkscape")
                    .join("extensions")
                    .join("inkstitch");
                if user_ext.exists() {
                    return true;
                }
            }

            // Check in application bundle
            let app_ext = inkscape_path.parent().and_then(|p| p.parent()).map(|p| {
                p.join("Resources")
                    .join("share")
                    .join("inkscape")
                    .join("extensions")
                    .join("inkstitch")
            });

            if let Some(path) = app_ext {
                return path.exists();
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Check in user's Inkscape profile
            if let Some(appdata) = dirs::data_dir() {
                let user_ext = appdata
                    .join("inkscape")
                    .join("extensions")
                    .join("inkstitch");
                if user_ext.exists() {
                    return true;
                }
            }

            // Check in program files
            let prog_ext = inkscape_path.parent().map(|p| p.parent()).map(|p| {
                p.expect("Failed to find Inkscape installation path")
                    .join("share")
                    .join("inkscape")
                    .join("extensions")
                    .join("inkstitch")
            });

            if let Some(path) = prog_ext {
                return path.exists();
            }
        }

        #[cfg(target_os = "linux")]
        fn find_inkstitch_extension(_inkscape_path: &Path) -> bool {
            // Check in user's home directory
            if let Some(home) = dirs::home_dir() {
                let user_ext = home
                    .join(".config")
                    .join("inkscape")
                    .join("extensions")
                    .join("inkstitch");
                if user_ext.exists() {
                    return true;
                }
            }

            // Check in system-wide installation
            let paths = [
                "/usr/share/inkscape/extensions/inkstitch",
                "/usr/local/share/inkscape/extensions/inkstitch",
            ];

            paths.iter().any(|path| Path::new(path).exists())
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FILE_FORMATS;

    #[test]
    #[ignore]
    fn test_formats_are_supported_by_inkstitch() {
        let known_formats: Vec<_> = FILE_FORMATS
            .iter()
            .map(|f| f.extension.to_lowercase())
            .collect();

        let unknown = SUPPORTED_READ_FORMATS
            .iter()
            .map(|ext| ext.to_lowercase())
            .filter(|ext| !known_formats.contains(ext))
            .collect::<Vec<_>>();

        assert!(
            unknown.is_empty(),
            "Found Ink/Stitch formats not defined in FILE_FORMATS: {:?}",
            unknown
        );
    }
}
