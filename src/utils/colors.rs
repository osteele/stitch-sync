/// Determine if the terminal likely has a dark background based on LS_COLORS
pub fn is_dark_theme(ls_colors: &str) -> bool {
    let entries = ls_colors.split(':');

    // Look for directory colors as they're usually most indicative
    for entry in entries {
        if entry.starts_with("di=") {
            let color_codes = entry.split('=').nth(1).unwrap_or("");
            let codes: Vec<&str> = color_codes.split(';').collect();

            // Check if any code is a foreground color (30-37 or 90-97)
            for code in codes {
                if let Ok(num) = code.parse::<u8>() {
                    match num {
                        30..=37 => {
                            // Dark colors (30-37) usually indicate light theme
                            return false;
                        }
                        90..=97 => {
                            // Bright colors (90-97) usually indicate dark theme
                            return true;
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    // Alternative detection: check if default text color is bright
    if let Some(rs) = ls_colors.split(':').find(|s| s.starts_with("rs=")) {
        let codes = rs.split('=').nth(1).unwrap_or("").split(';');
        for code in codes {
            if let Ok(num) = code.parse::<u8>() {
                if num >= 90 && num <= 97 {
                    return true; // Bright default text suggests dark theme
                }
            }
        }
    }

    // If we can't determine, check terminal background color if available
    if let Ok(term_bg) = std::env::var("COLORFGBG") {
        return term_bg.split(';').last().map_or(true, |bg| bg != "15");
    }

    // Default based on system preference if available
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            return output.status.success(); // "Dark" exists = dark mode
        }
    }

    false // Default to light theme if we can't determine
}

#[allow(dead_code)]
pub enum MessageType {
    Warning,
    Info,
    Success,
    Emphasis,
    Dimmed,
    Error,
}

pub fn get_contrasting_color(message_type: MessageType) -> crossterm::style::Color {
    let is_dark = if let Ok(ls_colors) = std::env::var("LS_COLORS") {
        is_dark_theme(&ls_colors)
    } else {
        true // default to dark theme
    };

    use crossterm::style::Color::*;
    match (message_type, is_dark) {
        (MessageType::Warning, true) => AnsiValue(178), // Light gold for dark theme
        (MessageType::Warning, false) => AnsiValue(94), // Darker gold for light theme

        (MessageType::Info, true) => AnsiValue(75), // Bright blue for dark theme
        (MessageType::Info, false) => AnsiValue(25), // Darker blue for light theme

        (MessageType::Success, true) => AnsiValue(46), // Bright green for dark theme
        (MessageType::Success, false) => AnsiValue(28), // Darker green for light theme

        (MessageType::Emphasis, true) => AnsiValue(255), // White for dark theme
        (MessageType::Emphasis, false) => AnsiValue(16), // Black for light theme

        (MessageType::Dimmed, true) => AnsiValue(242), // Light gray for dark theme
        (MessageType::Dimmed, false) => AnsiValue(246), // Darker gray for light theme

        (MessageType::Error, true) => AnsiValue(196), // Bright red for dark theme
        (MessageType::Error, false) => AnsiValue(124), // Darker red for light theme
    }
}
