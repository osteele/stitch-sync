use lazy_static::lazy_static;
use strsim::jaro_winkler;

use crate::utils::{prompt_from_list, prompt_yes_no, CsvReader};

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

    fn normalize_name(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    }

    pub fn find_by_name(name: &str) -> Option<Machine> {
        let normalized_search = Self::normalize_name(name);
        MACHINES
            .iter()
            .find(|machine| {
                let normalized_name = Self::normalize_name(&machine.name);
                let normalized_synonyms: Vec<String> = machine
                    .synonyms
                    .iter()
                    .map(|s| Self::normalize_name(s))
                    .collect();

                normalized_name == normalized_search
                    || normalized_synonyms.contains(&normalized_search)
            })
            .cloned()
    }

    /// Returns machines with names similar to the search term, sorted by similarity score
    /// Threshold is between 0.0 and 1.0, where 1.0 is an exact match
    pub fn find_similar_names(name: &str, threshold: f64) -> Vec<Machine> {
        let normalized_search = Self::normalize_name(name);
        let mut matches: Vec<(f64, Machine)> = MACHINES
            .iter()
            .filter_map(|machine| {
                // Check main name
                let name_score =
                    jaro_winkler(&normalized_search, &Self::normalize_name(&machine.name));

                // Check synonyms
                let synonym_score = machine
                    .synonyms
                    .iter()
                    .map(|s| jaro_winkler(&normalized_search, &Self::normalize_name(s)))
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);

                // Use the better score between name and synonyms
                let best_score = name_score.max(synonym_score);

                if best_score >= threshold {
                    Some((best_score, machine.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Sort by similarity score in descending order
        matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        matches.into_iter().map(|(_, machine)| machine).collect()
    }

    pub fn interactive_find_by_name(name: &str) -> Option<Machine> {
        if let Some(machine) = Self::find_by_name(name) {
            return Some(machine);
        }
        let similar_machines = Self::find_similar_names(name, 0.8);
        match similar_machines.len() {
            0 => None,
            1 => {
                println!(
                    "I found one machine that might match: {}",
                    similar_machines[0].name
                );
                println!("Use this?");
                if prompt_yes_no(Some(true)) {
                    Some(similar_machines[0].clone())
                } else {
                    None
                }
            }
            _ => {
                println!("Did you mean:");
                let names: Vec<String> = similar_machines.iter().map(|m| m.name.clone()).collect();
                let index = prompt_from_list(&names);
                index.map(|index| similar_machines[index].clone())
            }
        }
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

    #[test]
    fn test_get_machine_info_case_insensitive() {
        assert!(Machine::find_by_name("brother pe800").is_some());
        assert!(Machine::find_by_name("BROTHER PE800").is_some());
        assert!(Machine::find_by_name("Brother-PE800").is_some());
        assert!(Machine::find_by_name("Brother PE 800").is_some());
    }

    #[test]
    #[ignore]
    fn test_find_similar_names() {
        // Test exact match
        let results = Machine::find_similar_names("Brother PE800", 0.9);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Brother PE800");

        // Test close match
        let results = Machine::find_similar_names("Brother PE 80", 0.8);
        assert!(!results.is_empty());
        assert!(results.iter().any(|m| m.name == "Brother PE800"));

        // Test no matches with high threshold
        let results = Machine::find_similar_names("XYZ123", 0.9);
        assert!(results.is_empty());

        // Test partial name
        let results = Machine::find_similar_names("PE800", 0.7);
        assert!(!results.is_empty());
        assert!(results.iter().any(|m| m.name == "Brother PE800"));

        // Test with synonym
        let pe800 = Machine::find_by_name("Brother PE800").unwrap();
        if !pe800.synonyms.is_empty() {
            let results = Machine::find_similar_names(&pe800.synonyms[0], 0.8);
            assert!(!results.is_empty());
            assert!(results.iter().any(|m| m.name == "Brother PE800"));
        }
    }
}
