use csv::ReaderBuilder;
use lazy_static::lazy_static;
use std::io::Cursor;

use crate::utils::get_column_index;

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub formats: Vec<String>,
    pub usb_path: Option<String>,
    pub notes: Option<String>,
    pub design_size: Option<String>,
}

impl Machine {
    pub fn new(
        name: String,
        formats: Vec<&str>,
        usb_path: Option<String>,
        notes: Option<String>,
        design_size: Option<String>,
    ) -> Self {
        let formats = formats.iter().map(|f| f.to_string()).collect();
        Self {
            name,
            formats,
            usb_path,
            notes,
            design_size,
        }
    }

    pub fn find_by_name(name: &str) -> Option<Machine> {
        MACHINES
            .iter()
            .find(|machine| machine.name == name)
            .cloned()
    }
}

lazy_static! {
    pub static ref MACHINES: Vec<Machine> = {
        let csv_data = include_str!("./machines.csv");
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(Cursor::new(csv_data));

        let headers = reader.headers().unwrap();
        let name_idx = get_column_index(headers, "Machine Name").unwrap();
        let formats_idx = get_column_index(headers, "File Formats").unwrap();
        let usb_idx = get_column_index(headers, "USB Path").unwrap();
        let notes_idx = get_column_index(headers, "Notes").unwrap();
        let design_size_idx = get_column_index(headers, "Design Size").unwrap();

        let mut machines = Vec::new();
        for result in reader.records() {
            let record = result.unwrap();
            let formats: Vec<&str> = record[formats_idx].split(',').map(|s| s.trim()).collect();

            machines.push(Machine::new(
                record[name_idx].to_string(),
                formats,
                Some(record[usb_idx].to_string()).filter(|s| !s.is_empty()),
                Some(record[notes_idx].to_string()).filter(|s| !s.is_empty()),
                Some(record[design_size_idx].to_string()).filter(|s| !s.is_empty()),
            ));
        }
        machines
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_unique_machine_names() {
        let mut name_groups: HashMap<String, Vec<String>> = HashMap::new();

        MACHINES.iter().for_each(|m| {
            let normalized_name: String = m
                .name
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();

            name_groups
                .entry(normalized_name)
                .or_default()
                .push(m.name.clone());
        });

        let duplicates: Vec<_> = name_groups
            .into_values()
            .filter(|names| names.len() > 1)
            .collect();

        assert!(
            duplicates.is_empty(),
            "Found equivalent machine names: {:?}",
            duplicates
        );
    }

    #[test]
    fn test_get_machine_info() {
        assert!(Machine::find_by_name("Brother PE800").is_some());
        assert!(Machine::find_by_name("Nonexistent Machine").is_none());
    }
}
