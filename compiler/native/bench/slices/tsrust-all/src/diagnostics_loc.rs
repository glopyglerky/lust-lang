//! Go: diagnostics/loc_generated.go, and the localization part of
//! diagnostics/diagnostics.go (`(*Message).Localize`, `Localize`,
//! `getLocalizedMessages`, `Format`).
//!
//! PORT: the message catalogs are the Go files `diagnostics/loc/*.json.gz`,
//! copied unchanged to `crates/ts_goport/data/loc/` and embedded with
//! `include_bytes!` in place of Go `//go:embed`.
//! PORT: Go `sync.OnceValue` is a `OnceLock` per catalog, and the Go
//! `sync.Map` cache is a `Mutex` around a map.

use crate::prelude::*;

use crate::locale::{Locale, language};
use std::io::Read;
use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};

// Go: diagnostics/diagnostics.go:44 Key
// PORT: a key is a `&str` (`Message::key`); catalogs map `String` keys.

/// A localized message catalog: message key to localized text.
pub type LocaleMessages = FxHashMap<String, String>;

// Go: diagnostics/loc_generated.go:15 matcher

// Go: diagnostics/loc_generated.go:32 localeFuncs

// Go: diagnostics/loc_generated.go:49 loadLocaleData

/// Go: one `//go:embed loc/<locale>.json.gz` variable and its
/// `sync.OnceValue` loader.
macro_rules! locale_data {
    ($data:ident, $func:ident, $file:literal) => {
        static $data: &[u8] = include_bytes!(concat!("../data/loc/", $file));

        fn $func() -> &'static LocaleMessages {
            static MESSAGES: OnceLock<LocaleMessages> = OnceLock::new();
            MESSAGES.get_or_init(|| load_locale_data($data))
        }
    };
}

// Go: diagnostics/loc_generated.go:63 zhCNData, zhCN
// Go: diagnostics/loc_generated.go:70 zhTWData, zhTW
// Go: diagnostics/loc_generated.go:77 csCZData, csCZ
// Go: diagnostics/loc_generated.go:84 deDEData, deDE
// Go: diagnostics/loc_generated.go:91 esESData, esES
// Go: diagnostics/loc_generated.go:98 frFRData, frFR
// Go: diagnostics/loc_generated.go:105 itITData, itIT
// Go: diagnostics/loc_generated.go:112 jaJPData, jaJP
// Go: diagnostics/loc_generated.go:119 koKRData, koKR
// Go: diagnostics/loc_generated.go:126 plPLData, plPL
// Go: diagnostics/loc_generated.go:133 ptBRData, ptBR
// Go: diagnostics/loc_generated.go:140 ruRUData, ruRU
// Go: diagnostics/loc_generated.go:147 trTRData, trTR

// Go: diagnostics/diagnostics.go:67 (*Message).Localize

// Go: diagnostics/diagnostics.go:83 Localize

// Go: diagnostics/diagnostics.go:99 localizedMessagesCache
static LOCALIZED_MESSAGES_CACHE: LazyLock<
    Mutex<FxHashMap<language::Tag, Option<&'static LocaleMessages>>>,
> = LazyLock::new(|| Mutex::new(FxHashMap::default()));

// Go: diagnostics/diagnostics.go:101 getLocalizedMessages

// Go: diagnostics/diagnostics.go:127 placeholderRegexp
// Go: diagnostics/diagnostics.go:129 Format

// Go: diagnostics/diagnostics.go:148 StringifyArgs
// PORT: not ported; Rust callers pass `String` args (see `message_localize`).

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(s: &str) -> Locale {
        let (l, ok) = crate::locale::parse(s);
        assert!(ok, "{s} should parse");
        l
    }

    #[test]
    fn parse_reports_invalid_tags() {
        assert!(!crate::locale::parse("").1);
        assert!(!crate::locale::parse("xx").1);
        assert!(!crate::locale::parse("de-AB").1);
        assert!(!crate::locale::parse("toolongtagname").1);
        assert!(crate::locale::parse("de").1);
        assert!(crate::locale::parse("ja-jp").1);
        assert!(crate::locale::parse("en_US").1);
    }

    #[test]
    fn locales_match_like_tsgo() {
        // (tag, index in LOCALE_FUNCS or None for English), checked against
        // x/text v0.38.0 and tsgo-oracle --locale.
        let cases: [(&str, Option<usize>); 16] = [
            ("en-US", None),
            ("de", Some(4)),
            ("de-AT", Some(4)),
            ("zh-cn", Some(1)),
            ("zh-HK", Some(2)),
            ("zh-Hant", Some(2)),
            ("pt-PT", Some(11)),
            ("es-419", Some(5)),
            ("gl", Some(5)),
            ("gsw", Some(4)),
            ("be", Some(12)),
            ("und-JP", Some(8)),
            ("uk", None),
            ("sr", None),
            ("yue", None),
            ("x-foo", None),
        ];
        for (tag, want) in cases {
            let messages = get_localized_messages(&locale(tag).0);
            let want = want
                .and_then(|i| LOCALE_FUNCS[i])
                .map(|f| f() as *const LocaleMessages);
            let got = messages.map(|m| m as *const LocaleMessages);
            assert_eq!(got, want, "{tag}");
        }
    }

    #[test]
    fn localizes_messages() {
        let m = diag::Type_0_is_not_assignable_to_type_1;
        let args = args!["string", "number"];
        assert_eq!(
            message_localize(m, &locale("de"), &args),
            "Der Typ \"string\" kann dem Typ \"number\" nicht zugewiesen werden."
        );
        assert_eq!(
            message_localize(m, &locale("zh-cn"), &args),
            "不能将类型“string”分配给类型“number”。"
        );
        assert_eq!(
            message_localize(m, &crate::locale::DEFAULT, &args),
            "Type 'string' is not assignable to type 'number'."
        );
    }

    #[test]
    fn every_catalog_loads() {
        for f in LOCALE_FUNCS.iter().flatten() {
            assert!(!f().is_empty());
        }
    }

    #[test]
    fn format_matches_go_regexp() {
        let args = args!["a", "b"];
        assert_eq!(format("{0}{1}", &args), "ab");
        assert_eq!(format("{{0}} {x} {} {1", &args), "{a} {x} {} {1");
        assert_eq!(format("{0}", &[]), "{0}");
        // Go `strings.ToValidUTF8`: a run of invalid bytes (port form units,
        // see `scanner_util::GO_STRING_MARKER`) becomes one U+FFFD, and a
        // real U+FDD0 stays.
        let args = args!["a\u{FDD0}\u{10F7FE}\u{FDD0}\u{10F7FF}b", "\u{FDD0}\u{FDD0}"];
        assert_eq!(format("{0} {1}", &args), "a\u{FFFD}b \u{FDD0}\u{FDD0}");
    }
}
