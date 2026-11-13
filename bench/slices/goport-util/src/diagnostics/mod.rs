//! TypeScript diagnostic messages and rendered diagnostic instances. This
//! was the `ts_diagnostics` crate; `catalog.rs` comes from
//! `tools/ts_diagnostics_codegen`.

mod catalog;
mod effect_catalog;

use std::{error::Error, fmt};


/// Go `diagnostics.Category`, an `int32`. The four named values are
/// TypeScript's diagnostic severity categories. A bad `.tsbuildinfo` can
/// give any other value. Go keeps it: it sorts as an int and is written to
/// the build info again as it was read. Go panics only when the diagnostic
/// is printed ("Unhandled diagnostic category").
// PORT: a newtype with associated consts, so `Category::Error` reads as the
// Go `diagnostics.CategoryError`.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Category(pub i32);

#[allow(non_upper_case_globals)]
impl Category {
    pub const Warning: Self = Self(0);
    pub const Error: Self = Self(1);
    pub const Suggestion: Self = Self(2);
    pub const Message: Self = Self(3);

    // Go: diagnostics/diagnostics.go:26 (Category).Name
}







/// A missing numbered diagnostic formatting argument.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatError {
    pub code: u32,
    pub argument_index: usize,
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "diagnostic TS{} requires formatting argument {}",
            self.code, self.argument_index
        )
    }
}

impl Error for FormatError {}


/// The first Effect diagnostic code. Effect messages (377000 to 377999) are
/// in `EFFECT_CATALOG`, not in the generated TypeScript catalog.
pub const EFFECT_CODE_START: u32 = 377_000;


/// The key of a message with `text` and `code`: Go's
/// `internal/diagnostics/generate.go` `convertPropertyName`. Each `*`,
/// `/` and `:` becomes a word, each other character that is not a letter
/// or digit becomes `_`; runs of `_` become one; leading `_` before a
/// non-digit and one trailing `_` go; the result is cut to 100 bytes and
/// gets `_<code>`.
// PORT: not in Go at run time. The wasm build makes its keys with it.
#[cfg(any(test, target_family = "wasm"))]
fn message_key(text: &str, code: u32) -> String {
    let mut name = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '*' => name.push_str("_Asterisk"),
            '/' => name.push_str("_Slash"),
            ':' => name.push_str("_Colon"),
            c if c.is_alphabetic() || c.is_numeric() => name.push(c),
            _ => name.push('_'),
        }
    }
    // `_+` -> `_`
    let mut collapsed = String::with_capacity(name.len());
    for c in name.chars() {
        if !(c == '_' && collapsed.ends_with('_')) {
            collapsed.push(c);
        }
    }
    // `^_+(\D)` -> `$1` (after the collapse there is at most one `_`)
    let mut key = match collapsed.strip_prefix('_') {
        Some(rest) if rest.chars().next().is_some_and(|c| !c.is_ascii_digit()) => rest.to_string(),
        _ => collapsed,
    };
    // `_$` -> ``
    if key.ends_with('_') {
        key.pop();
    }
    if key.len() > 100 {
        key.truncate(100);
    }
    format!("{key}_{code}")
}

/// The catalog text reader of the wasm build (`parts/goport_util/build.rs`),
/// for its test.
#[cfg(test)]
#[path = "../../parts/goport_util/catalog_texts.rs"]
mod catalog_texts;

#[cfg(test)]
mod tests {
    use super::{CATALOG, Category, Diagnostic, message_by_code, message_by_key, message_key};

    /// The wasm build packs the texts that `catalog_texts` reads from the
    /// catalog source; they must be the texts of the catalog.
    #[test]
    fn catalog_texts_reads_every_text() {
        let texts = super::catalog_texts::catalog_texts(include_str!("catalog.rs"));
        let texts: Vec<&str> = texts.split_terminator('\0').collect();
        let want: Vec<&str> = CATALOG.iter().map(|message| message.text()).collect();
        assert_eq!(texts, want);
    }

    /// The wasm build makes each key from its text (`message_key`); it
    /// must give the generated key of every message.
    #[test]
    fn message_key_makes_every_generated_key() {
        for message in CATALOG {
            assert_eq!(
                message_key(message.text(), message.code()),
                message.key(),
                "{}",
                message.code()
            );
        }
    }

    #[test]
    fn generated_catalog_is_complete_and_sorted() {
        assert_eq!(CATALOG.len(), 2_215);
        assert!(
            CATALOG
                .windows(2)
                .all(|pair| pair[0].code() < pair[1].code())
        );
    }

    #[test]
    fn representative_catalog_entries_match_typescript() {
        let unterminated = message_by_code(1002).unwrap();
        assert_eq!(unterminated.category(), Category::Error);
        assert_eq!(unterminated.text(), "Unterminated string literal.");
        assert_eq!(unterminated.key(), "Unterminated_string_literal_1002");

        let unused = message_by_code(6133).unwrap();
        assert_eq!(
            unused.text(),
            "'{0}' is declared but its value is never read."
        );
        assert!(unused.reports_unnecessary());

        let deprecated = message_by_code(6385).unwrap();
        assert_eq!(deprecated.category(), Category::Suggestion);
        assert!(deprecated.reports_deprecated());

        // Go #63987 moved the native-only messages from 100000 to TS codes.
        let native = message_by_code(6933).unwrap();
        assert_eq!(native.category(), Category::Message);
        assert_eq!(native.text(), "Do not print diagnostics.");
        assert!(message_by_code(100_000).is_none());
    }

    #[test]
    fn newly_exposed_checker_and_option_diagnostics_match_typescript() {
        let cases: &[(u32, &[&str], &str)] = &[
            (2349, &[], "This expression is not callable."),
            (
                5052,
                &["checkJs", "allowJs"],
                "Option 'checkJs' cannot be specified without specifying option 'allowJs'.",
            ),
            (
                6504,
                &["src/caf\u{00e9}.js"],
                "File 'src/caf\u{00e9}.js' is a JavaScript file. Did you mean to enable the 'allowJs' option?",
            ),
            (
                2874,
                &["React"],
                "This JSX tag requires 'React' to be in scope, but it could not be found.",
            ),
            (
                2875,
                &["react/jsx-runtime"],
                "This JSX tag requires the module path 'react/jsx-runtime' to exist, but none could be found. Make sure you have types for the appropriate package installed.",
            ),
        ];

        for &(code, arguments, expected) in cases {
            let message = message_by_code(code).expect("diagnostic exists in the pinned catalog");
            assert_eq!(message.category(), Category::Error);
            let diagnostic = Diagnostic::with_arguments(message, arguments.iter().copied());
            assert_eq!(diagnostic.render().unwrap(), expected, "TS{code}");
        }
    }

    #[test]
    fn ambient_declaration_diagnostics_match_pinned_typescript_records() {
        let cases: &[(u32, &str, &[&str], &str)] = &[
            (
                1039,
                "Initializers_are_not_allowed_in_ambient_contexts_1039",
                &[],
                "Initializers are not allowed in ambient contexts.",
            ),
            (
                1046,
                "Top_level_declarations_in_d_ts_files_must_start_with_either_a_declare_or_export_modifier_1046",
                &[],
                "Top-level declarations in .d.ts files must start with either a 'declare' or 'export' modifier.",
            ),
            (
                7010,
                "_0_which_lacks_return_type_annotation_implicitly_has_an_1_return_type_7010",
                &["foo", "any"],
                "'foo', which lacks return-type annotation, implicitly has an 'any' return type.",
            ),
        ];

        for &(code, key, arguments, expected) in cases {
            let message = message_by_code(code).expect("ambient diagnostic is in the catalog");
            assert_eq!(message.key(), key, "TS{code}");
            assert_eq!(message.category(), Category::Error, "TS{code}");
            assert_eq!(
                Diagnostic::with_arguments(message, arguments.iter().copied())
                    .render()
                    .unwrap(),
                expected,
                "TS{code}"
            );
        }
    }

    #[test]
    fn directive_and_related_diagnostics_keep_upstream_categories_and_flags() {
        let unused_directive = message_by_code(2578).unwrap();
        assert_eq!(unused_directive.category(), Category::Error);
        assert_eq!(
            unused_directive.text(),
            "Unused '@ts-expect-error' directive."
        );
        assert!(!unused_directive.reports_unnecessary());
        assert!(!unused_directive.reports_deprecated());

        let missing_argument = message_by_code(6210).unwrap();
        assert_eq!(missing_argument.category(), Category::Message);
        assert_eq!(
            Diagnostic::with_arguments(missing_argument, ["right"])
                .render()
                .unwrap(),
            "An argument for 'right' was not provided."
        );

        let property_origin = message_by_code(6500).unwrap();
        assert_eq!(property_origin.category(), Category::Message);
        assert_eq!(
            Diagnostic::with_arguments(property_origin, ["value", "Target"])
                .render()
                .unwrap(),
            "The expected type comes from property 'value' which is declared here on type 'Target'"
        );

        let elided = message_by_code(2202).unwrap();
        assert!(elided.elided_in_compatibility_pyramid());
        assert!(!missing_argument.elided_in_compatibility_pyramid());
    }

    #[test]
    fn diagnostics_render_numbered_arguments() {
        let message = message_by_code(1007).unwrap();
        let diagnostic = Diagnostic::with_arguments(message, ["{", "}"]);
        assert_eq!(
            diagnostic.render().unwrap(),
            "The parser expected to find a '}' to match the '{' token here."
        );
        assert_eq!(diagnostic.code(), 1007);
    }

    #[test]
    fn diagnostics_render_placeholders_inside_literal_braces() {
        let diagnostic =
            Diagnostic::with_arguments(message_by_code(2613).unwrap(), ["./module", "namedExport"]);
        assert_eq!(
            diagnostic.render().unwrap(),
            "Module './module' has no default export. Did you mean to use 'import { namedExport } from ./module' instead?"
        );
    }

    #[test]
    fn diagnostics_report_missing_numbered_arguments() {
        let diagnostic = Diagnostic::with_arguments(message_by_code(1007).unwrap(), ["{"]);
        let error = diagnostic.render().unwrap_err();
        assert_eq!(error.code, 1007);
        assert_eq!(error.argument_index, 1);
    }

    #[test]
    fn diagnostics_render_indented_message_details() {
        let message = message_by_code(2322).unwrap();
        let diagnostic = Diagnostic::with_arguments(message, ["source", "target"])
            .with_details(["  Types of parameters are incompatible."]);
        assert_eq!(
            diagnostic.render().unwrap(),
            concat!(
                "Type 'source' is not assignable to type 'target'.\n",
                "  Types of parameters are incompatible.",
            )
        );
    }

    #[test]
    fn lookup_by_key_uses_generated_key() {
        assert_eq!(
            message_by_key("Identifier_expected_1003").unwrap().code(),
            1003
        );
        assert!(message_by_code(u32::MAX).is_none());
    }
}
