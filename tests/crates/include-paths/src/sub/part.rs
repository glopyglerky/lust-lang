// This file enters through include! in main.rs; its own include is
// relative to this file's directory.
pub fn inner_text() -> &'static str {
    include_str!("inner.txt")
}
