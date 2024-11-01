use lazy_static::lazy_static;

use crate::utils::CsvReader;

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub synonyms: Vec<String>,
    pub formats: Vec<String>,
    pub usb_path: Option<String>,
    pub notes: Option<String>,
    pub design_size: Option<String>,
}

impl Machine {
    pub fn new(
        name: String,
        synonyms: Vec<String>,
        formats: Vec<String>,
        usb_path: Option<String>,
        notes: Option<String>,
        design_size: Option<String>,
    ) -> Self {
        Self {
            name,
            synonyms,
            formats,
            usb_path: usb_path.filter(|s| !s.is_empty()),
            notes: notes.filter(|s| !s.is_empty()),
            design_size: design_size.filter(|s| !s.is_empty()),
        }
    }

    pub fn find_by_name(name: &str) -> Option<Machine> {
        MACHINES
            .iter()
            .find(|machine| machine.name == name || machine.synonyms.contains(&name.to_string()))
            .cloned()
    }
}

lazy_static! {
    pub static ref MACHINES: Vec<Machine> = {
        let csv_data = include_str!("./machines.csv");
        let mut reader = CsvReader::from_str(csv_data).unwrap();

        reader
            .iter_records()
            .map(|result| {
                let record = result.unwrap();
                Machine::new(
                    record.get("Machine Name").unwrap().to_string(),
                    record.get_vec("Synonyms", ',').unwrap_or_default(),
                    record.get_vec("File Formats", ',').unwrap(),
                    record.get("USB Path").map(ToString::to_string),
                    record.get("Notes").map(ToString::to_string),
                    record.get("Design Size").map(ToString::to_string),
                )
            })
            .collect()
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
