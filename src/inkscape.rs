use std::path::{Path, PathBuf};
use which::which;

pub struct InkscapeInfo {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub has_inkstitch: bool,
}

pub fn find_inkscape() -> Option<InkscapeInfo> {
    // First try the PATH as it works on all platforms
    if let Ok(path) = which("inkscape") {
        let has_inkstitch = find_inkstitch_extension(&path);
        return Some(InkscapeInfo {
            path,
            has_inkstitch,
        });
    }

    // Platform-specific locations
    #[cfg(target_os = "macos")]
    {
        let app_path = PathBuf::from("/Applications/Inkscape.app/Contents/MacOS/inkscape");
        if app_path.exists() {
            let has_inkstitch = find_inkstitch_extension(&app_path);
            return Some(InkscapeInfo {
                path: app_path,
                has_inkstitch,
            });
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
                let has_inkstitch = find_inkstitch_extension(&path);
                return Some(InkscapeInfo {
                    path,
                    has_inkstitch,
                });
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
                let has_inkstitch = find_inkstitch_extension(&path);
                return Some(InkscapeInfo {
                    path,
                    has_inkstitch,
                });
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
            p.join("share")
                .join("inkscape")
                .join("extensions")
                .join("inkstitch")
        });

        if let Some(path) = prog_ext {
            return path.exists();
        }
    }

    #[cfg(target_os = "linux")]
    {
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

        for path in paths {
            if Path::new(path).exists() {
                return true;
            }
        }
    }

    false
}
