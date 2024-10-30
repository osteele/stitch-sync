use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum FileFormat {
    Art,     // Bernina
    Csd,     // Singer EU/Poem/Huskygram
    Dst,     // Tajima
    Exp,     // Melco/Bernina
    Fhe,     // Singer Futura
    Hus,     // Husqvarna Viking
    Jef,     // Janome
    JefPlus, // Janome
    Jpx,     // Janome
    Pcd,     // Pfaff
    Pcm,     // Pfaff
    Pcs,     // Pfaff
    Pec,     // Brother (subset of PES)
    Pes,     // Brother
    Psw,     // Singer Professional Sew Ware
    Sew,     // Janome/Elna
    Vip,     // Viking/Pfaff
    Vp3,     // Husqvarna/Viking/Pfaff
    Xxx,     // Singer
    Zsk,     // ZSK Embroidery
}

pub struct FileFormatInfo {
    #[allow(dead_code)]
    pub name: &'static str,
    pub extension: &'static str,
    pub manufacturer: &'static str,
    pub notes: Option<&'static str>,
}

impl FileFormat {
    pub fn get_info(&self) -> FileFormatInfo {
        match self {
            FileFormat::Art => FileFormatInfo {
                name: "Bernina Embroidery Format",
                extension: "art",
                manufacturer: "Bernina",
                notes: None,
            },
            FileFormat::Csd => FileFormatInfo {
                name: "Singer Compatible Design",
                extension: "csd",
                manufacturer: "Singer",
                notes: Some("Used by older Singer EU/Poem/Huskygram machines"),
            },
            FileFormat::Dst => FileFormatInfo {
                name: "Tajima",
                extension: "dst",
                manufacturer: "Tajima",
                notes: Some(
                    "Industry standard format, widely supported by home and commercial machines",
                ),
            },
            FileFormat::Exp => FileFormatInfo {
                name: "Melco Expanded",
                extension: "exp",
                manufacturer: "Melco/Bravo",
                notes: Some("Used by Bernina and Melco machines"),
            },
            FileFormat::Fhe => FileFormatInfo {
                name: "Singer Futura",
                extension: "fhe",
                manufacturer: "Singer",
                notes: Some("Native format for Singer Futura machines"),
            },
            FileFormat::Hus => FileFormatInfo {
                name: "Husqvarna Viking",
                extension: "hus",
                manufacturer: "Husqvarna/Viking",
                notes: None,
            },
            FileFormat::Jef => FileFormatInfo {
                name: "Janome Embroidery Format",
                extension: "jef",
                manufacturer: "Janome",
                notes: None,
            },
            FileFormat::JefPlus => FileFormatInfo {
                name: "Extended Janome Embroidery Format",
                extension: "jef+",
                manufacturer: "Janome",
                notes: Some("Enhanced version of JEF for larger designs and more advanced edits"),
            },
            FileFormat::Jpx => FileFormatInfo {
                name: "Janome Extended",
                extension: "jpx",
                manufacturer: "Janome",
                notes: Some(
                    "Janome proprietary format that includes stitch data and background images",
                ),
            },
            FileFormat::Pcd => FileFormatInfo {
                name: "Pfaff",
                extension: "pcd",
                manufacturer: "Pfaff",
                notes: None,
            },
            FileFormat::Pcm => FileFormatInfo {
                name: "Pfaff",
                extension: "pcm",
                manufacturer: "Pfaff",
                notes: None,
            },
            FileFormat::Pcs => FileFormatInfo {
                name: "Pfaff",
                extension: "pcs",
                manufacturer: "Pfaff",
                notes: None,
            },

            FileFormat::Pec => FileFormatInfo {
                name: "Brother (subset of PES)",
                extension: "pec",
                manufacturer: "Brother",
                notes: None,
            },
            FileFormat::Pes => FileFormatInfo {
                name: "Brother Embroidery Format",
                extension: "pes",
                manufacturer: "Brother",
                notes: Some("Brother/Babylock format, popular for home machines"),
            },
            FileFormat::Vip => FileFormatInfo {
                name: "Viking/Pfaff",
                extension: "vip",
                manufacturer: "Viking/Pfaff",
                notes: Some("Legacy format"),
            },
            FileFormat::Vp3 => FileFormatInfo {
                name: "Viking/Pfaff Phase 3",
                extension: "vp3",
                manufacturer: "Viking/Pfaff",
                notes: Some("Current format for Viking and Pfaff machines"),
            },
            FileFormat::Xxx => FileFormatInfo {
                name: "Singer",
                extension: "xxx",
                manufacturer: "Singer",
                notes: None,
            },

            FileFormat::Psw => FileFormatInfo {
                name: "Singer Professional Sew Ware",
                extension: "psw",
                manufacturer: "Singer",
                notes: None,
            },
            FileFormat::Sew => FileFormatInfo {
                name: "Janome/Elna",
                extension: "sew",
                manufacturer: "Janome/Elna",
                notes: None,
            },
            FileFormat::Zsk => FileFormatInfo {
                name: "ZSK Embroidery",
                extension: "zsk",
                manufacturer: "ZSK",
                notes: None,
            },
        }
    }
}

lazy_static! {
    pub static ref ALL_FORMATS: [FileFormat; 15] = [
        FileFormat::Dst,
        FileFormat::Exp,
        FileFormat::Jef,
        FileFormat::JefPlus,
        FileFormat::Pes,
        FileFormat::Vip,
        FileFormat::Vp3,
        FileFormat::Xxx,
        FileFormat::Csd,
        FileFormat::Fhe,
        FileFormat::Pcs,
        FileFormat::Pec,
        FileFormat::Psw,
        FileFormat::Sew,
        FileFormat::Zsk,
    ];
}
