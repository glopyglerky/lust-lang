//! Port of Effect-TS/tsgo `etscore` at `@effect/tsgo@0.46.1`: the
//! `@effect/language-service` plugin options, severities and the command
//! line mode flag.

use crate::indexmap::IndexMap;
use std::sync::atomic::{AtomicBool, Ordering};

// Go: etscore/consts.go
pub const EFFECT_PLUGIN_NAME: &str = "@effect/language-service";

// Go: etscore/version_generated.go EffectVersion
/// The `@effect/tsgo` release this port follows. Update it with the port.
/// Build info written with the rules on records it (`build_info_version`).
pub const EFFECT_VERSION: &str = "0.46.1";

// Go: etscore/severity.go
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Severity {
    #[default]
    Off,
    Warning,
    Error,
    Suggestion,
    Message,
    /// Used in directives only, not in tsconfig.
    SkipFile,
}

impl Severity {
    // Go: etscore/severity.go ParseSeverity
    #[must_use]
    pub fn parse(s: &str) -> Severity {
        match s.trim().to_lowercase().as_str() {
            "off" => Severity::Off,
            "warning" | "warn" => Severity::Warning,
            "error" => Severity::Error,
            "suggestion" => Severity::Suggestion,
            "message" => Severity::Message,
            "skip-file" => Severity::SkipFile,
            // default to error for unknown values
            _ => Severity::Error,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Off => "off",
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Suggestion => "suggestion",
            Severity::Message => "message",
            Severity::SkipFile => "skip-file",
        }
    }

    #[must_use]
    pub fn is_off(self) -> bool {
        self == Severity::Off || self == Severity::SkipFile
    }

    fn visibility_rank(self) -> i32 {
        match self {
            Severity::Error => 4,
            Severity::Warning => 3,
            Severity::Suggestion => 2,
            Severity::Message => 1,
            _ => 0,
        }
    }

    #[must_use]
    pub fn at_least_as_visible_as(self, min: Severity) -> bool {
        self.visibility_rank() >= min.visibility_rank()
    }
}

// Go: etscore/climode.go
static COMMAND_LINE_MODE: AtomicBool = AtomicBool::new(false);

/// Go `EnterCommandLineMode`: sets the mode and returns a guard that restores
/// the previous value when dropped.
#[must_use]
pub fn enter_command_line_mode() -> CommandLineModeGuard {
    CommandLineModeGuard {
        prev: COMMAND_LINE_MODE.swap(true, Ordering::SeqCst),
    }
}

pub struct CommandLineModeGuard {
    prev: bool,
}

impl Drop for CommandLineModeGuard {
    fn drop(&mut self) {
        COMMAND_LINE_MODE.store(self.prev, Ordering::SeqCst);
    }
}

#[must_use]
pub fn is_command_line_mode() -> bool {
    COMMAND_LINE_MODE.load(Ordering::SeqCst)
}

// Go: etscore/options.go
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyPattern {
    pub target: String,
    pub pattern: String,
    pub skip_leading_path: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TopLevelNamedReexportsMode {
    /// Go `""`, the zero value.
    #[default]
    Unset,
    Ignore,
    Follow,
}





pub const EFFECT_FN_SPAN: &str = "span";
pub const EFFECT_FN_UNTRACED: &str = "untraced";
pub const EFFECT_FN_NO_SPAN: &str = "no-span";
pub const EFFECT_FN_INFERRED_SPAN: &str = "inferred-span";
pub const EFFECT_FN_SUGGESTED_SPAN: &str = "suggested-span";

const DEFAULT_EFFECT_FN: &[&str] = &[EFFECT_FN_SPAN];

// Go: etscore/options.go DiagnosticsEnabled



// Go: etscore/options.go DefaultKeyPatterns
#[must_use]
pub fn default_key_patterns() -> Vec<KeyPattern> {
    vec![
        KeyPattern {
            target: "service".into(),
            pattern: "default".into(),
            skip_leading_path: vec!["src/".into()],
        },
        KeyPattern {
            target: "custom".into(),
            pattern: "default".into(),
            skip_leading_path: vec!["src/".into()],
        },
    ]
}

// Go: etscore/options.go normalizePackageName
#[must_use]
pub fn normalize_package_name(name: &str) -> String {
    name.trim().to_lowercase()
}

// Go: etscore/options.go normalizePackageList
#[must_use]
pub fn normalize_package_list(packages: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(packages.len());
    for p in packages {
        let normalized = normalize_package_name(p);
        if !normalized.is_empty() {
            result.push(normalized);
        }
    }
    result
}

// Go: etscore/options_parser.go








// Go: etscore/options_parser.go ParseFromPlugins










// Go: the JSON form of `EffectPluginOptions` (its `json:",omitzero"` tags),
// which patch 028 writes to `.tsbuildinfo` as `effect`.



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_severity_matches_go() {
        for (input, want) in [
            ("off", Severity::Off),
            ("WARN", Severity::Warning),
            ("warning", Severity::Warning),
            ("Error", Severity::Error),
            ("suggestion", Severity::Suggestion),
            ("message", Severity::Message),
            ("skip-file", Severity::SkipFile),
            ("unknown", Severity::Error),
            ("", Severity::Error),
        ] {
            assert_eq!(Severity::parse(input), want, "{input}");
        }
    }

    #[test]
    fn build_info_value_round_trips() {
        let plugin = CompilerOptionsValue::List(vec![CompilerOptionsValue::Map(
            [
                (
                    "name",
                    CompilerOptionsValue::String(EFFECT_PLUGIN_NAME.into()),
                ),
                (
                    "namespaceImportPackages",
                    CompilerOptionsValue::List(vec![CompilerOptionsValue::String("effect".into())]),
                ),
                (
                    "diagnosticSeverity",
                    CompilerOptionsValue::Map(
                        [(
                            "globalDate".to_string(),
                            CompilerOptionsValue::String("error".into()),
                        )]
                        .into_iter()
                        .collect(),
                    ),
                ),
                (
                    "overrides",
                    CompilerOptionsValue::List(vec![CompilerOptionsValue::Map(
                        [
                            (
                                "include".to_string(),
                                CompilerOptionsValue::List(vec![CompilerOptionsValue::String(
                                    "**/*.test.ts".into(),
                                )]),
                            ),
                            (
                                "options".to_string(),
                                CompilerOptionsValue::Map(
                                    [(
                                        "diagnosticSeverity".to_string(),
                                        CompilerOptionsValue::Map(
                                            [(
                                                "preferSchemaOverJson".to_string(),
                                                CompilerOptionsValue::String("off".into()),
                                            )]
                                            .into_iter()
                                            .collect(),
                                        ),
                                    )]
                                    .into_iter()
                                    .collect(),
                                ),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    )]),
                ),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
        )]);
        let options = parse_from_plugins(&plugin).expect("plugin");
        assert!(options.include_suggestions_in_tsc);
        assert_eq!(
            EffectPluginOptions::from_value(&options.to_value()),
            Some(options)
        );
    }
}
