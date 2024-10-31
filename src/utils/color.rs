pub fn colorize(text: &str, color_code: &str) -> String {
    if std::env::var_os("NO_COLOR").is_none() {
        format!("\x1b[{}m{}\x1b[0m", color_code, text)
    } else {
        text.to_string()
    }
}

pub fn red(text: &str) -> String {
    colorize(text, "31")
}
