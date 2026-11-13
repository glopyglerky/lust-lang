// expect: L0820
// A block type item named like a primitive or std prelude type: valid
// Rust that lust rejects on purpose (L0820), since check reads those
// names by text in value paths (`u8::default()` would call the builtin).
fn main() {
    #[derive(Debug, Default)]
    struct u8 { a: i32 }
    let s = u8::default();
    println!("{:?}", s);
}
