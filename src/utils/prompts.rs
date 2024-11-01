use std::io::{stdout, Write};

pub fn prompt_yes_no(default: Option<bool>) -> bool {
    loop {
        let input = prompt_input("(y/n) ");
        match input.to_lowercase().trim() {
            "y" => return true,
            "n" => return false,
            "" if default.is_some() => return default.unwrap(),
            _ => {}
        }
    }
}

pub fn prompt_input(prompt: &str) -> String {
    print!("{}", prompt);
    stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn prompt_from_list(list: &Vec<String>) -> Option<usize> {
    for (i, machine) in list.iter().enumerate() {
        println!("  {}. {}", i + 1, machine);
    }
    loop {
        let input = prompt_input("Enter number, or 'q' to cancel: ");
        if input.to_lowercase().trim() == "q" {
            return None;
        }
        let index: usize = input.parse().ok()?;
        if index > 0 && index <= list.len() {
            return Some(index - 1);
        }
        println!("Please enter a number between 1 and {}", list.len());
    }
}
