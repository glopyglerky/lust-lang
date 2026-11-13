//! Kernel extraction of ts-rust diagnostics/mod.rs: the Category and
//! Message types and FormatError, copied verbatim except one marked
//! edit. The catalog itself is not copied; crate::diag holds the
//! extracted entries the scanner uses. The wasm-only branches are kept
//! as in the original and compile out on this target.

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
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Suggestion => "suggestion",
            Self::Message => "message",
            _ => panic!("Unhandled diagnostic category") // KERNEL-EDIT: was crate::core::go_panic,
        }
    }
}

/// Prints a named value as the old enum did (`Error`), and any other value
/// as `Category(999)`.
impl fmt::Debug for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Warning => formatter.write_str("Warning"),
            Self::Error => formatter.write_str("Error"),
            Self::Suggestion => formatter.write_str("Suggestion"),
            Self::Message => formatter.write_str("Message"),
            Self(raw) => write!(formatter, "Category({raw})"),
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// A catalog entry from TypeScript's diagnosticMessages.json.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Message {
    code: u32,
    category: Category,
    /// "" in a catalog entry of the wasm build (`Message::catalog`).
    key: &'static str,
    /// "" in a catalog entry of the wasm build (`Message::catalog`).
    text: &'static str,
    reports_unnecessary: bool,
    elided_in_compatibility_pyramid: bool,
    reports_deprecated: bool,
}

impl Message {
    /// A message made outside the catalog: `ast::NIL_MESSAGE` and the ad
    /// hoc messages (code 0, key ""), and the messages that Go removed
    /// (`diag.rs`). It keeps its key and text on every target.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(
        code: u32,
        category: Category,
        key: &'static str,
        text: &'static str,
        reports_unnecessary: bool,
        elided_in_compatibility_pyramid: bool,
        reports_deprecated: bool,
    ) -> Self {
        // The wasm build tells catalog entries by their code with no key.
        assert!(
            code == 0 || !key.is_empty(),
            "a message with a code has a key"
        );
        Self {
            code,
            category,
            key,
            text,
            reports_unnecessary,
            elided_in_compatibility_pyramid,
            reports_deprecated,
        }
    }

    /// An entry of the generated `CATALOG`. The wasm build leaves out its
    /// key (144 KB for all) and text (151 KB): `key` makes the key from the
    /// text, and `text` reads the texts that `parts/goport_util/build.rs`
    /// packed.
    #[doc(hidden)]
    #[must_use]
    pub const fn catalog(
        code: u32,
        category: Category,
        key: &'static str,
        text: &'static str,
        reports_unnecessary: bool,
        elided_in_compatibility_pyramid: bool,
        reports_deprecated: bool,
    ) -> Self {
        let packed = cfg!(target_family = "wasm");
        Self {
            code,
            category,
            key: if packed { "" } else { key },
            text: if packed { "" } else { text },
            reports_unnecessary,
            elided_in_compatibility_pyramid,
            reports_deprecated,
        }
    }

    /// wasm: true for a catalog entry, which has a code and no key.
    #[cfg(target_family = "wasm")]
    const fn packed(self) -> bool {
        self.code != 0 && self.key.is_empty()
    }

    #[must_use]
    pub const fn code(self) -> u32 {
        self.code
    }

    #[must_use]
    pub const fn category(self) -> Category {
        self.category
    }

    #[cfg(not(target_family = "wasm"))]
    #[must_use]
    pub const fn key(self) -> &'static str {
        self.key
    }

    /// wasm: the key. A catalog entry makes it from its text as Go's
    /// generator does (`message_key`), once.
    #[cfg(target_family = "wasm")]
    #[must_use]
    pub fn key(self) -> &'static str {
        use std::collections::HashMap;
        use std::sync::{Mutex, PoisonError};
        if !self.packed() {
            return self.key;
        }
        static KEYS: Mutex<Option<HashMap<u32, &'static str>>> = Mutex::new(None);
        let mut keys = KEYS.lock().unwrap_or_else(PoisonError::into_inner);
        keys.get_or_insert_with(HashMap::new)
            .entry(self.code)
            .or_insert_with(|| Box::leak(message_key(self.text(), self.code).into_boxed_str()))
    }

    #[cfg(not(target_family = "wasm"))]
    #[must_use]
    pub const fn text(self) -> &'static str {
        self.text
    }

    /// wasm: the text. A catalog entry reads it from the packed texts of
    /// the catalog, which the first read unpacks.
    #[cfg(target_family = "wasm")]
    #[must_use]
    pub fn text(self) -> &'static str {
        if !self.packed() {
            return self.text;
        }
        static TEXTS: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
        let texts = TEXTS.get_or_init(|| {
            let packed = include_bytes!(concat!(env!("OUT_DIR"), "/diagnostic_texts.lzma"));
            let texts: &'static str =
                Box::leak(crate::frontend::bundled::unpack(packed).into_boxed_str());
            let texts: Vec<&'static str> = texts.split_terminator('\0').collect();
            assert_eq!(
                texts.len(),
                CATALOG.len(),
                "a packed text for each catalog entry"
            );
            texts
        });
        let index = CATALOG
            .binary_search_by_key(&self.code, |message| message.code)
            .expect("a catalog message");
        texts[index]
    }

    #[must_use]
    pub const fn reports_unnecessary(self) -> bool {
        self.reports_unnecessary
    }

    #[must_use]
    pub const fn elided_in_compatibility_pyramid(self) -> bool {
        self.elided_in_compatibility_pyramid
    }

    #[must_use]
    pub const fn reports_deprecated(self) -> bool {
        self.reports_deprecated
    }

    /// Formats numbered placeholders such as {0} and {1}.
    ///
    /// # Errors
    ///
    /// Returns an error if the message references an argument that was not supplied.
    pub fn format(self, arguments: &[String]) -> Result<String, FormatError> {
        let text = self.text();
        if arguments.is_empty() {
            return Ok(text.to_owned());
        }

        let mut output = String::with_capacity(text.len());
        let mut remaining = text;
        while let Some(open) = remaining.find('{') {
            output.push_str(&remaining[..open]);
            remaining = &remaining[open + 1..];
            let digits = remaining.bytes().take_while(u8::is_ascii_digit).count();
            if digits == 0 || remaining.as_bytes().get(digits) != Some(&b'}') {
                output.push('{');
                continue;
            }

            let index_text = &remaining[..digits];
            let Ok(index) = index_text.parse::<usize>() else {
                output.push('{');
                continue;
            };
            let Some(argument) = arguments.get(index) else {
                return Err(FormatError {
                    code: self.code,
                    argument_index: index,
                });
            };
            output.push_str(argument);
            remaining = &remaining[digits + 1..];
        }
        output.push_str(remaining);
        Ok(output)
    }
}

/// One diagnostic occurrence with formatting arguments.
#[derive(Clone, Debug, Eq, PartialEq)]

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
