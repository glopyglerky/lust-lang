// macro: locale_data
// source: crates/ts_goport/src/diagnostics_loc.rs:83
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::sync::OnceLock;

pub struct LocaleMessages { pub text: String }
pub fn load_locale_data(data: &[u8]) -> LocaleMessages {
    LocaleMessages { text: String::from_utf8_lossy(data).trim().to_string() }
}

macro_rules! locale_data {
    ($data:ident, $func:ident, $file:literal) => {
        static $data: &[u8] = include_bytes!(concat!("../data/loc/", $file));

        fn $func() -> &'static LocaleMessages {
            static MESSAGES: OnceLock<LocaleMessages> = OnceLock::new();
            MESSAGES.get_or_init(|| load_locale_data($data))
        }
    };
}

locale_data!(EN_DATA, en, "en.json");

fn main() {
    println!("locale='{}' bytes={} same={}", en().text, EN_DATA.len(), std::ptr::eq(en(), en()));
}
