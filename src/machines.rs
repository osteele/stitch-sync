use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::file_formats::FileFormat;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineInfo {
    pub name: String,
    pub formats: Vec<FileFormat>,
    pub usb_path: Option<String>,
    pub notes: Option<String>,
}

impl MachineInfo {
    pub fn new(
        name: String,
        formats: Vec<FileFormat>,
        usb_path: Option<String>,
        notes: Option<String>,
    ) -> Self {
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
            "Janome MB-4".into(),
            vec![FileFormat::Jef, FileFormat::JefPlus, FileFormat::Dst],
            Some("EMB".into()),
            Some(
                "4-needle machine with RCS unit; built-in memory capacity of 3MB.".into(),
            ),
        ),
        MachineInfo::new(
            "Brother PE800".into(),
            vec![FileFormat::Pes],
            Some("EMB/Embf".into()),
            Some("Accepts up to 5x7 inch designs".into()),
        ),
        MachineInfo::new(
            "Brother PE535".into(),
            vec![FileFormat::Pes],
            Some("EMB/Embf".into()),
            Some("Accepts up to 4x4 inch designs".into()),
        ),
        MachineInfo::new(
            "Brother SE1900".into(),
            vec![FileFormat::Pes],
            Some("EMB/Embf".into()),
            Some(
                "Accepts up to 5x7 inch designs. Combination sewing/embroidery machine.".into(),
            ),
        ),
        MachineInfo::new(
            "Brother SE600".into(),
            vec![FileFormat::Pes],
            Some("EMB/Embf".into()),
            Some(
                "Accepts up to 4x4 inch designs. Combination sewing/embroidery machine.".into(),
            ),
        ),
        // Janome Machines
        MachineInfo::new(
            "Janome 200E".into(),
            vec![FileFormat::Jef],
            Some("EMB/Embf".into()),
            None,
        ),
        MachineInfo::new(
            "Janome 300E".into(),
            vec![FileFormat::Jef],
            Some("Embf5".into()),
            None,
        ),
        MachineInfo::new(
            "Janome 350E".into(),
            vec![FileFormat::Jef],
            Some("Embf5/MyDesign".into()),
            None,
        ),
        MachineInfo::new(
            "Janome 9500/9700".into(),
            vec![FileFormat::Jef],
            Some("Embf5".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MB4".into(),
            vec![FileFormat::Jef],
            Some("EMB/Embf".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC400E".into(),
            vec![FileFormat::Jef],
            Some("EMB".into()),
            Some("Accepts up to 7.9x7.9 inch designs".into()),
        ),
        MachineInfo::new(
            "Janome MC500E".into(),
            vec![FileFormat::Jef],
            Some("EMB".into()),
            Some("Accepts up to 7.9x11 inch designs".into()),
        ),
        MachineInfo::new(
            "Janome MC9900".into(),
            vec![FileFormat::Jef, FileFormat::Dst],
            Some("EMB".into()),
            Some("Accepts up to 6.7x7.9 inch designs".into()),
        ),
        MachineInfo::new(
            "Janome MC10001".into(),
            vec![FileFormat::Jef],
            Some("Embf5".into()),
            Some("Supports Embf5 through Embf16 folders".into()),
        ),
        MachineInfo::new(
            "Janome MC11000".into(),
            vec![FileFormat::Jef, FileFormat::Dst],
            Some("EMB/Embf".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC12000".into(),
            vec![FileFormat::Jef],
            Some("EMB/Embf".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC15000".into(),
            vec![FileFormat::Jef],
            Some("EMB/Embf".into()),
            Some("Main folder not required, EMB/Embf are optional paths".into()),
        ),
        MachineInfo::new(
            "Janome MC9900".into(),
            vec![FileFormat::Jef, FileFormat::Dst],
            Some("EMB/Embf".into()),
            None,
        ),
        // Bernette Machines
        MachineInfo::new(
            "Bernette B70".into(),
            vec![FileFormat::Exp],
            None,
            Some("Accepts up to 6x10 inch designs".into()),
        ),
        MachineInfo::new(
            "Bernette B79".into(),
            vec![FileFormat::Exp],
            None,
            Some("Accepts up to 6x10 inch designs".into()),
        ),
        // Bernina Machines
        MachineInfo::new(
            "Bernina 770".into(),
            vec![FileFormat::Exp],
            None,
            Some("Accepts up to 9.5x6 inch designs".into()),
        ),
        MachineInfo::new(
            "Bernina 790".into(),
            vec![FileFormat::Exp],
            None,
            Some("Accepts up to 15.7x10.2 inch designs".into()),
        ),
        // Singer Machines
        MachineInfo::new(
            "Singer Futura CE-100".into(),
            vec![
                FileFormat::Csd, FileFormat::Xxx, FileFormat::Hus,
                    FileFormat::Dst, FileFormat::Zsk, FileFormat::Pcs
            ],
            None,
            Some("Accepts up to 4.50x6.75 inch designs".into()),
        ),
        MachineInfo::new(
            "Singer Legacy SE300".into(),
            vec![FileFormat::Xxx],
            None,
            Some("Accepts up to 10.25x6 inch designs".into()),
        ),
        MachineInfo::new(
            "Singer Legacy SE340".into(),
            vec![FileFormat::Xxx],
            None,
            Some("Accepts up to 7x12 inch designs".into()),
        ),
        MachineInfo::new(
            "Singer Quantum XL-1000".into(),
            vec![FileFormat::Xxx, FileFormat::Dst, FileFormat::Zsk],
            None,
            Some("Accepts up to 5.50x9.5 inch designs. Max 15 color stops.".into()),
        ),
        MachineInfo::new(
            "Singer Quantum XL-5000".into(),
            vec![
                FileFormat::Xxx, FileFormat::Dst, FileFormat::Zsk,
                FileFormat::Pes, FileFormat::Pcs, FileFormat::Psw
            ],
            None,
            Some("Accepts up to 5x7 inch designs. Max 15 color stops.".into()),
        ),
        MachineInfo::new(
            "Singer Futura CE-250".into(),
            vec![
                FileFormat::Fhe, FileFormat::Xxx, FileFormat::Psw,
                FileFormat::Pec, FileFormat::Pes, FileFormat::Hus,
                FileFormat::Sew, FileFormat::Exp, FileFormat::Dst,
                FileFormat::Pcs
            ],
            None,
            Some("Accepts up to 4.5x6.75 inch designs. Connects directly to computer.".into()),
        ),
        // Husqvarna Viking Machines
        MachineInfo::new(
            "Husqvarna Designer Epic2".into(),
            vec![FileFormat::Vp3, FileFormat::Vip],
            None,
            Some("Accepts up to 360x360mm designs".into()),
        ),
        MachineInfo::new(
            "Husqvarna Designer Ruby90".into(),
            vec![FileFormat::Vp3, FileFormat::Vip],
            None,
            Some("Accepts up to 360x260mm designs".into()),
        ),
        // Pfaff Machines
        MachineInfo::new(
            "Pfaff Creative Icon2".into(),
            vec![FileFormat::Vp3],
            None,
            Some("Accepts up to 360x360mm designs".into()),
        ),
        MachineInfo::new(
            "Pfaff Creative 4".into(),
            vec![FileFormat::Vp3],
            None,
            Some("Accepts up to 360x260mm designs".into()),
        ),
        MachineInfo::new(
            "Janome MB-7".into(),
            vec![FileFormat::Jef, FileFormat::JefPlus, FileFormat::Dst],
            Some("EMB".into()),
            Some("7-needle embroidery machine".into()),
        ),
        MachineInfo::new(
            "Janome Memory Craft 550E".into(),
            vec![FileFormat::Jef, FileFormat::Dst],
            Some("EMB".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC300".into(),
            vec![FileFormat::Jef, FileFormat::Dst],
            Some("EMB".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC350".into(),
            vec![FileFormat::Jef, FileFormat::Dst],
            Some("EMB".into()),
            None,
        ),
        MachineInfo::new(
            "Janome MC9500".into(),
            vec![FileFormat::Jef, FileFormat::Dst],
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
