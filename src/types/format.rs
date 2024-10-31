use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct FileFormat {
    #[allow(dead_code)]
    pub name: &'static str,
    pub extension: &'static str,
    pub manufacturer: &'static str,
    pub notes: Option<&'static str>,
}

lazy_static! {
    pub static ref FILE_FORMATS: Vec<FileFormat> = vec![
        FileFormat {
            name: "Bernina Embroidery Format",
            extension: "art",
            manufacturer: "Bernina",
            notes: None,
        },
        FileFormat {
            name: "Singer Compatible Design",
            extension: "csd",
            manufacturer: "Singer",
            notes: Some("Used by older Singer EU/Poem/Huskygram machines"),
        },
        FileFormat {
            name: "Tajima",
            extension: "dst",
            manufacturer: "Tajima",
            notes: Some(
                "Industry standard format, widely supported by home and commercial machines",
            ),
        },
        FileFormat {
            name: "Melco Expanded",
            extension: "exp",
            manufacturer: "Melco/Bravo",
            notes: Some("Used by Bernina and Melco machines"),
        },
        FileFormat {
            name: "Singer Futura",
            extension: "fhe",
            manufacturer: "Singer",
            notes: Some("Native format for Singer Futura machines"),
        },
        FileFormat {
            name: "Husqvarna Viking",
            extension: "hus",
            manufacturer: "Husqvarna/Viking",
            notes: None,
        },
        FileFormat {
            name: "Janome Embroidery Format",
            extension: "jef",
            manufacturer: "Janome",
            notes: None,
        },
        FileFormat {
            name: "Extended Janome Embroidery Format",
            extension: "jef+",
            manufacturer: "Janome",
            notes: Some("Enhanced version of JEF for larger designs and more advanced edits"),
        },
        FileFormat {
            name: "Janome Extended",
            extension: "jpx",
            manufacturer: "Janome",
            notes: Some(
                "Janome proprietary format that includes stitch data and background images",
            ),
        },
        FileFormat {
            name: "Pfaff PC-Designer",
            extension: "pcd",
            manufacturer: "Pfaff",
            notes: None,
        },
        FileFormat {
            name: "Pfaff Embroidery Design Files",
            extension: "pcm",
            manufacturer: "Pfaff",
            notes: None,
        },
        FileFormat {
            name: "Pfaff",
            extension: "pcs",
            manufacturer: "Pfaff",
            notes: None,
        },
        FileFormat {
            name: "Brother (subset of PES)",
            extension: "pec",
            manufacturer: "Brother",
            notes: None,
        },
        FileFormat {
            name: "Brother Embroidery Format",
            extension: "pes",
            manufacturer: "Brother",
            notes: Some("Brother/Babylock format, popular for home machines"),
        },
        FileFormat {
            name: "Viking/Pfaff",
            extension: "vip",
            manufacturer: "Viking/Pfaff",
            notes: Some("Legacy format"),
        },
        FileFormat {
            name: "Viking/Pfaff Phase 3",
            extension: "vp3",
            manufacturer: "Viking/Pfaff",
            notes: Some("Current format for Viking and Pfaff machines"),
        },
        FileFormat {
            name: "Singer",
            extension: "xxx",
            manufacturer: "Singer",
            notes: None,
        },
        FileFormat {
            name: "Singer Professional Sew Ware",
            extension: "psw",
            manufacturer: "Singer",
            notes: None,
        },
        FileFormat {
            name: "Janome/Elna",
            extension: "sew",
            manufacturer: "Janome/Elna",
            notes: None,
        },
        FileFormat {
            name: "ZSK Embroidery",
            extension: "zsk",
            manufacturer: "ZSK",
            notes: None,
        },
    ];
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
            *ext_counts.entry(f.extension).or_insert(0) += 1;
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
            *name_counts.entry(f.name).or_insert(0) += 1;
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
