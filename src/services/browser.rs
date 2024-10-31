use std::process::Command;

pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", url]).spawn().ok();
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().ok();
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn().ok();
    }
}
