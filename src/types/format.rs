use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FileFormat {
    #[allow(dead_code)]
    pub name: String,
    pub extension: String,
    pub manufacturer: String,
    pub notes: Option<String>,
}

lazy_static! {
    pub static ref FILE_FORMATS: Vec<FileFormat> = {
        let yaml_content = include_str!("./formats.yaml");
        serde_yaml::from_str(yaml_content).expect("Failed to parse formats.yaml")
    };
}

impl FileFormat {
    pub fn find_by_extension(extension: &str) -> Option<&'static FileFormat> {
        FILE_FORMATS.iter().find(|f| f.extension == extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_unique_extensions() {
        let mut ext_counts: HashMap<&str, usize> = HashMap::new();
        FILE_FORMATS.iter().for_each(|f| {
            *ext_counts.entry(f.extension.as_str()).or_insert(0) += 1;
        });

        let duplicates: Vec<_> = ext_counts
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(ext, _)| ext)
            .collect();

        assert!(
            duplicates.is_empty(),
            "Found duplicate file extensions: {:?}",
            duplicates
        );
    }

    #[test]
    fn test_unique_names() {
        let mut name_counts: HashMap<&str, usize> = HashMap::new();
        FILE_FORMATS.iter().for_each(|f| {
            *name_counts.entry(f.name.as_str()).or_insert(0) += 1;
        });

        let duplicates: Vec<_> = name_counts
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(name, _)| name)
            .collect();

        assert!(
            duplicates.is_empty(),
            "Found duplicate file format names: {:?}",
            duplicates
        );
    }

    #[test]
    fn test_find_by_extension() {
        assert!(FileFormat::find_by_extension("dst").is_some());
        assert!(FileFormat::find_by_extension("nonexistent").is_none());
    }
}
