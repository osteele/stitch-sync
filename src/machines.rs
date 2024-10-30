use lazy_static::lazy_static;

use crate::file_formats::{self, FileFormat};

#[derive(Debug, Clone)]
pub struct MachineInfo {
    pub name: String,
    pub formats: Vec<&'static FileFormat>,
    pub usb_path: Option<String>,
    pub notes: Option<String>,
}

impl MachineInfo {
    pub fn new(
        name: String,
        formats: Vec<&str>,
        usb_path: Option<String>,
        notes: Option<String>,
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
        }
    }
}

lazy_static! {
    pub static ref MACHINES: Vec<MachineInfo> = vec![
        // Brother Machines
        MachineInfo::new(
            "Brother PE800".into(),
            vec!["pes"],
            Some("EMB/Embf".into()),
            Some("Accepts up to 5x7 inch designs".into()),
        ),
        MachineInfo::new(
            "Brother PE535".into(),
            vec!["pes"],
            Some("EMB/Embf".into()),
            Some("Accepts up to 4x4 inch designs".into()),
        ),
        MachineInfo::new(
            "Brother SE1900".into(),
            vec!["pes"],
            Some("EMB/Embf".into()),
            Some(
                "Accepts up to 5x7 inch designs. Combination sewing/embroidery machine.".into(),
            ),
        ),
        MachineInfo::new(
            "Brother SE600".into(),
            vec!["pes"],
            Some("EMB/Embf".into()),
            Some(
                "Accepts up to 4x4 inch designs. Combination sewing/embroidery machine.".into(),
            ),
        ),
        // Janome Machines
        MachineInfo::new(
            "Janome 200E".into(),
            vec!["jef"],
            Some("EMB/Embf".into()),
            None,
        ),
        MachineInfo::new(
            "Janome 300E".into(),
            vec!["jef"],
            Some("Embf5".into()),
            None,
        ),
        MachineInfo::new(
            "Janome 350E".into(),
            vec!["jef"],
            Some("Embf5/MyDesign".into()),
            None,
        ),
        MachineInfo::new(
            "Janome 9500/9700".into(),
            vec!["jef"],
            Some("Embf5".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MB4".into(),
            vec!["jef", "jef+", "dst"],
            Some("EMB".into()),
            Some(
                "4-needle machine with RCS unit; built-in memory capacity of 3MB.".into(),
            ),
        ),
        MachineInfo::new(
            "Janome MC400E".into(),
            vec!["jef"],
            Some("EMB".into()),
            Some("Accepts up to 7.9x7.9 inch designs".into()),
        ),
        MachineInfo::new(
            "Janome MC500E".into(),
            vec!["jef"],
            Some("EMB".into()),
            Some("Accepts up to 7.9x11 inch designs".into()),
        ),
        MachineInfo::new(
            "Janome MC9900".into(),
            vec!["jef", "dst"],
            Some("EMB/Embf".into()),
            Some("Accepts up to 6.7x7.9 inch designs".into()),
        ),
        MachineInfo::new(
            "Janome MC10001".into(),
            vec!["jef"],
            Some("Embf5".into()),
            Some("Supports Embf5 through Embf16 folders".into()),
        ),
        MachineInfo::new(
            "Janome MC11000".into(),
            vec!["jef", "dst"],
            Some("EMB/Embf".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC12000".into(),
            vec!["jef", "dst"],
            Some("EMB/Embf".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC15000".into(),
            vec!["jef", "dst"],
            Some("EMB/Embf".into()),
            Some("Main folder not required, EMB/Embf are optional paths".into()),
        ),
        // Bernette Machines
        MachineInfo::new(
            "Bernette B70".into(),
            vec!["exp"],
            None,
            Some("Accepts up to 6x10 inch designs".into()),
        ),
        MachineInfo::new(
            "Bernette B79".into(),
            vec!["exp"],
            None,
            Some("Accepts up to 6x10 inch designs".into()),
        ),
        // Bernina Machines
        MachineInfo::new(
            "Bernina 770".into(),
            vec!["exp"],
            None,
            Some("Accepts up to 9.5x6 inch designs".into()),
        ),
        MachineInfo::new(
            "Bernina 790".into(),
            vec!["exp"],
            None,
            Some("Accepts up to 15.7x10.2 inch designs".into()),
        ),
        // Singer Machines
        MachineInfo::new(
            "Singer Futura CE-100".into(),
            vec![   "csd", "xxx", "hus", "dst", "zsk", "pcs"],
            None,
            Some("Accepts up to 4.50x6.75 inch designs".into()),
        ),
        MachineInfo::new(
            "Singer Legacy SE300".into(),
            vec!["xxx"],
            None,
            Some("Accepts up to 10.25x6 inch designs".into()),
        ),
        MachineInfo::new(
            "Singer Legacy SE340".into(),
            vec!["xxx"],
            None,
            Some("Accepts up to 7x12 inch designs".into()),
        ),
        MachineInfo::new(
            "Singer Quantum XL-1000".into(),
            vec!["xxx", "dst", "zsk"],
            None,
            Some("Accepts up to 5.50x9.5 inch designs. Max 15 color stops.".into()),
        ),
        MachineInfo::new(
            "Singer Quantum XL-5000".into(),
            vec![
                "xxx", "dst", "zsk", "pes", "pcs", "psw"
            ],
            None,
            Some("Accepts up to 5x7 inch designs. Max 15 color stops.".into()),
        ),
        MachineInfo::new(
            "Singer Futura CE-250".into(),
            vec![
                "fhe", "xxx", "psw", "pec", "pes", "hus",
                "sew", "exp", "dst", "pcs"
            ],
            None,
            Some("Accepts up to 4.5x6.75 inch designs. Connects directly to computer.".into()),
        ),
        // Husqvarna Viking Machines
        MachineInfo::new(
            "Husqvarna Designer Epic2".into(),
            vec!["vp3", "vip"],
            None,
            Some("Accepts up to 360x360mm designs".into()),
        ),
        MachineInfo::new(
            "Husqvarna Designer Ruby90".into(),
            vec!["vp3", "vip"],
            None,
            Some("Accepts up to 360x260mm designs".into()),
        ),
        // Pfaff Machines
        MachineInfo::new(
            "Pfaff Creative Icon2".into(),
            vec!["vp3"],
            None,
            Some("Accepts up to 360x360mm designs".into()),
        ),
        MachineInfo::new(
            "Pfaff Creative 4".into(),
            vec!["vp3"],
            None,
            Some("Accepts up to 360x260mm designs".into()),
        ),
        MachineInfo::new(
            "Janome MB-7".into(),
            vec!["jef", "jef+", "dst"],
            Some("EMB".into()),
            Some("7-needle embroidery machine".into()),
        ),
        MachineInfo::new(
            "Janome Memory Craft 550E".into(),
            vec!["jef", "dst"],
            Some("EMB".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC300".into(),
            vec!["jef", "dst"],
            Some("EMB".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC350".into(),
            vec!["jef", "dst"],
            Some("EMB".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC9500".into(),
            vec!["jef", "dst"],
            Some("Embf5".into()),
            None,
        ),
    ];
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
