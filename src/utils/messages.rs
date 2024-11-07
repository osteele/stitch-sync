#[macro_export]
macro_rules! print_error {
    ($fmt:literal, $($arg:tt)*) => {{
        use crossterm::style::Stylize;
        use crate::utils::colors::{get_contrasting_color, MessageType};
        let msg = format!($fmt, $($arg)*);
        println!("{} ❌", msg.with(get_contrasting_color(MessageType::Error)))
    }};
    ($fmt:literal) => {{
        use crossterm::style::Stylize;
        use crate::utils::colors::{get_contrasting_color, MessageType};
        println!("{} ❌", $fmt.with(get_contrasting_color(MessageType::Error)))
    }};
}

#[macro_export]
macro_rules! print_notice {
    ($fmt:literal, $($arg:tt)*) => {{
        use crossterm::style::Stylize;
        use crate::utils::colors::{get_contrasting_color, MessageType};
        let msg = format!($fmt, $($arg)*);
        println!("{} 📝", msg.with(get_contrasting_color(MessageType::Success)))
    }};
    ($fmt:literal) => {{
        use crossterm::style::Stylize;
        use crate::utils::colors::{get_contrasting_color, MessageType};
        println!("{} 📝", $fmt.with(get_contrasting_color(MessageType::Success)))
    }};
}
