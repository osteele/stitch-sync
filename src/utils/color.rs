#[macro_export]
macro_rules! print_error {
    ($fmt:literal, $($arg:tt)*) => {{
        use crossterm::style::Stylize;
        println!("{} ❌", format!($fmt, $($arg)*).red())
    }};
    ($fmt:literal) => {{
        use crossterm::style::Stylize;
        println!("{} ❌", $fmt.red())
    }};
}
