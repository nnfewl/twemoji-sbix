pub const FAMILY_NAME: &str = "Apple Color Emoji";
pub const UNITS_PER_EM: u16 = 2048;
pub const ADVANCE_WIDTH: u16 = 2550;
pub const ASCENT: i16 = 1900;
pub const DESCENT: i16 = 500;

pub struct Preset {
    pub sizes: &'static [u16],
    pub description: &'static str,
}

pub const FULL: Preset = Preset {
    sizes: &[20, 26, 32, 40, 48, 52, 64, 96, 160],
    description: "All 9 strikes, matches Apple Color Emoji (~90 MB)",
};

pub const OPTIMAL: Preset = Preset {
    sizes: &[32, 64, 128],
    description: "3 strikes for terminal use on retina displays (~30 MB)",
};

pub const MINIMAL: Preset = Preset {
    sizes: &[64],
    description: "Single strike, smallest possible (~10 MB)",
};

pub fn get_preset(name: &str) -> Option<&'static Preset> {
    match name {
        "full" => Some(&FULL),
        "optimal" => Some(&OPTIMAL),
        "minimal" => Some(&MINIMAL),
        _ => None,
    }
}

pub fn list_presets() -> String {
    let presets = [("full", &FULL), ("optimal", &OPTIMAL), ("minimal", &MINIMAL)];
    presets
        .iter()
        .map(|(name, p)| {
            let default = if *name == "optimal" { " (default)" } else { "" };
            let sizes: Vec<String> = p.sizes.iter().map(|s| s.to_string()).collect();
            format!("  {}{}: [{}] — {}", name, default, sizes.join(", "), p.description)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
