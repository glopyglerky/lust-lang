// expect: L0104
// The escape_* iterator types do not exist in lust yet. The escape
// behavior itself is exact where lust uses it (Debug of char and str):
// see unicode_props_hash in tests/pass.
fn main() {
    let s: String = '\u{301}'.escape_debug().collect();
    println!("{}", s);
}
