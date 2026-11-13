//! Port of Go `core/compileroptions.go`, `core/tristate.go`,
//! `core/languagevariant.go` and `core/scriptkind.go` (plus their stringers).
//!
//! The Go enum types (`ScriptTarget`, `ModuleKind`, `ModuleResolutionKind`,
//! `ModuleDetectionKind`, `NewLineKind`, `JsxEmit`, `LanguageVariant`,
//! `ScriptKind`) are defined in `crate::flags` with the Go values. Their Go
//! methods are added here as inherent impls.

use crate::frontend::tspath;
use crate::prelude::*;

// ---------------------------------------------------------------------------
// core/tristate.go
// ---------------------------------------------------------------------------

/// Go `core.Tristate`. Values match Go (`TSUnknown` = 0, `TSFalse` = 1,
/// `TSTrue` = 2).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Tristate {
    #[default]
    Unknown = 0,
    False = 1,
    True = 2,
}

impl Tristate {
    // Go: core/tristate.go:15 IsTrue
    #[must_use]
    pub fn is_true(self) -> bool {
        self == Tristate::True
    }

    // Go: core/tristate.go:19 IsTrueOrUnknown
    #[must_use]
    pub fn is_true_or_unknown(self) -> bool {
        self == Tristate::True || self == Tristate::Unknown
    }

    // Go: core/tristate.go:23 IsFalse
    #[must_use]
    pub fn is_false(self) -> bool {
        self == Tristate::False
    }

    // Go: core/tristate.go:27 IsFalseOrUnknown
    #[must_use]
    pub fn is_false_or_unknown(self) -> bool {
        self == Tristate::False || self == Tristate::Unknown
    }

    // Go: core/tristate.go:31 IsUnknown
    #[must_use]
    pub fn is_unknown(self) -> bool {
        self == Tristate::Unknown
    }

    // Go: core/tristate.go:35 DefaultIfUnknown
    #[must_use]
    pub fn default_if_unknown(self, value: Tristate) -> Tristate {
        if self == Tristate::Unknown {
            return value;
        }
        self
    }

    // Go: core/tristate.go:42 UnmarshalJSON
    pub fn unmarshal_json(&mut self, data: &[u8]) {
        *self = match data {
            b"true" => Tristate::True,
            b"false" => Tristate::False,
            _ => Tristate::Unknown,
        };
    }

    // Go: core/tristate.go:54 MarshalJSON

    // Go: core/tristate_stringer_generated.go String
    #[must_use]
    pub fn string(self) -> String {
        match self {
            Tristate::Unknown => "TSUnknown",
            Tristate::False => "TSFalse",
            Tristate::True => "TSTrue",
        }
        .to_string()
    }
}

impl std::fmt::Display for Tristate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string())
    }
}

// Go: core/tristate.go:65 BoolToTristate
#[must_use]
pub fn bool_to_tristate(b: bool) -> Tristate {
    if b {
        return Tristate::True;
    }
    Tristate::False
}

// ---------------------------------------------------------------------------
// core/compileroptions.go
// ---------------------------------------------------------------------------

// Go: core/compileroptions.go:14 PluginImport (ts#64397)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginImport {
    pub name: String,
}



// Go: core/compileroptions.go:178 EmptyCompilerOptions


impl ModuleKind {
    // Go: core/compileroptions.go:430 ResolutionModeESM
    // PORT: Go `ResolutionMode` is an alias of `ModuleKind`, so
    // `core.ResolutionModeESM` is also reachable as `ResolutionMode::ESM`.
    // `ResolutionModeNone` and `ResolutionModeCommonJS` are
    // `ResolutionMode::NONE` and `ResolutionMode::COMMON_JS`.
    pub const ESM: Self = Self::ES_NEXT;

    // Go: core/compileroptions.go:415 IsNonNodeESM
    #[must_use]
    pub fn is_non_node_esm(self) -> bool {
        self >= ModuleKind::ES2015 && self <= ModuleKind::ES_NEXT
    }

    // Go: core/compileroptions.go:419 SupportsImportAttributes
    #[must_use]
    pub fn supports_import_attributes(self) -> bool {
        ModuleKind::NODE18 <= self && self <= ModuleKind::NODE_NEXT
            || self == ModuleKind::PRESERVE
            || self == ModuleKind::ES_NEXT
    }

    // Go: core/modulekind_stringer_generated.go String
}


/// Go `core.ResolutionMode` (`ModuleKindNone | ModuleKindCommonJS | ModuleKindESNext`).
// Go: core/compileroptions.go:425 ResolutionMode
pub type ResolutionMode = ModuleKind;

// Go: core/compileroptions.go:428 ResolutionModeNone
pub const RESOLUTION_MODE_NONE: ResolutionMode = ModuleKind::NONE;
// Go: core/compileroptions.go:429 ResolutionModeCommonJS
pub const RESOLUTION_MODE_COMMON_JS: ResolutionMode = ModuleKind::COMMON_JS;
// Go: core/compileroptions.go:430 ResolutionModeESM
pub const RESOLUTION_MODE_ESM: ResolutionMode = ModuleKind::ES_NEXT;

// Go: core/compileroptions.go:451 ModuleKindToModuleResolutionKind

impl ModuleResolutionKind {
    // Go: core/compileroptions.go:457 String
    // We don't use stringer on this for now, because these values
    // are user-facing in --traceResolution, and stringer currently
    // lacks the ability to remove the "ModuleResolutionKind" prefix
    // when generating code for multiple types into the same output
    // file. Additionally, since there's no TS equivalent of
    // `ModuleResolutionKindUnknown`, we want to panic on that case,
    // as it probably represents a mistake when porting TS to Go.
}


// Go: core/compileroptions.go:490 GetNewLineKind
#[must_use]
pub fn get_new_line_kind(s: &str) -> NewLineKind {
    match s {
        "\r\n" => NewLineKind::CRLF,
        "\n" => NewLineKind::LF,
        _ => NewLineKind::NONE,
    }
}

impl NewLineKind {
    // Go: core/compileroptions.go:501 GetNewLineCharacter
}

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

impl JsxEmit {
    // Go: core/compileroptions.go:538 String
}


// ---------------------------------------------------------------------------
// core/languagevariant.go, core/scriptkind.go
// ---------------------------------------------------------------------------
// The types and constants live in `crate::flags` (`LanguageVariant`,
// `ScriptKind`). Only the generated stringers are ported here.

impl LanguageVariant {
    // Go: core/languagevariant_stringer_generated.go String
}


impl ScriptKind {
    // Go: core/scriptkind_stringer_generated.go:26 String
    // tsgo#4712: values 5 (formerly ScriptKindExternal) and 7 (formerly
    // ScriptKindDeferred) are reserved and print as "ScriptKind(5)" and
    // "ScriptKind(7)".
}


// ---------------------------------------------------------------------------
// tspath helpers used above. Private so they do not clash with a tspath port
// elsewhere in the crate. `GetDirectoryPath` and `GetBaseFileName` are the
// `frontend::tspath` ports.
// ---------------------------------------------------------------------------

// Go: tspath/extension.go:121 GetDeclarationFileExtension

// Go: tspath/extension.go:113 IsDeclarationFileName

// Go: tspath/path.go:1116 ForEachAncestorDirectory
// PORT: the callback returns only `stop`; the only caller here has no result.
