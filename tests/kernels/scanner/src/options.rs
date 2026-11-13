//! Kernel extraction of the ts-rust options.rs items the scanner
//! uses: ScriptTarget::string and its Display, copied verbatim.

use crate::flags::ScriptTarget;

impl ScriptTarget {
    // Go: core/scripttarget_stringer_generated.go String
    #[must_use]
    pub fn string(self) -> String {
        let name = match self.0 {
            0 => "None",
            1 => "ES5",
            2 => "ES2015",
            3 => "ES2016",
            4 => "ES2017",
            5 => "ES2018",
            6 => "ES2019",
            7 => "ES2020",
            8 => "ES2021",
            9 => "ES2022",
            10 => "ES2023",
            11 => "ES2024",
            12 => "ES2025",
            13 => "ES2026",
            99 => "ESNext",
            100 => "JSON",
            _ => return format!("ScriptTarget({})", self.0),
        };
        name.to_string()
    }
}

impl std::fmt::Display for ScriptTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string())
    }
}
