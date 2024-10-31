use csv::ReaderBuilder;
use lazy_static::lazy_static;
use std::io::Cursor;

use crate::{
    file_formats::{self, FileFormat},
    utils::get_column_index,
};

#[derive(Debug, Clone)]
pub struct MachineInfo {
    pub name: String,
    pub formats: Vec<&'static FileFormat>,
    pub usb_path: Option<String>,
    pub notes: Option<String>,
    pub design_size: Option<String>,
}

impl MachineInfo {
    pub fn new(
        name: String,
        formats: Vec<&str>,
        usb_path: Option<String>,
        notes: Option<String>,
        design_size: Option<String>,
    ) -> Self {
        let formats = formats
            .iter()
            .map(|f| file_formats::find_by_extension(f).unwrap())
            .collect();
        Self {
            name,
            formats,
            usb_path,
            notes,
            design_size,
        }
    }
}

lazy_static! {
    pub static ref MACHINES: Vec<MachineInfo> = {
        let csv_data = include_str!("./assets/machines.csv");
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

            machines.push(MachineInfo::new(
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

pub fn get_machine_info(name: &str) -> Option<MachineInfo> {
    MACHINES
        .iter()
        .find(|machine| machine.name == name)
        .cloned()
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
        assert!(get_machine_info("Brother PE800").is_some());
        assert!(get_machine_info("Nonexistent Machine").is_none());
    }
}
