// Go: internal/typeparser/discover.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// DiscoveredPackage represents a package found in the program's source files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscoveredPackage {
    pub name: String,
    pub version: Option<String>,
    pub depends_on_effect: bool,
    pub package_directory: String,
}

/// packageKey is used for deduplication of discovered packages.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackageKey {
    pub name: String,
    pub version: String,
    pub has_ver: bool,
}

/// EffectMajorVersion represents the detected major version of the Effect library.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EffectMajorVersion {
    #[default]
    Unknown = 0,
    V3 = 3,
    V4 = 4,
}

impl EffectMajorVersion {
    /// String returns the string representation of the Effect major version.
    #[must_use]
    pub fn string(self) -> &'static str {
        match self {
            EffectMajorVersion::V3 => "v3",
            EffectMajorVersion::V4 => "v4",
            _ => "unknown",
        }
    }
}

impl std::fmt::Display for EffectMajorVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.string())
    }
}

