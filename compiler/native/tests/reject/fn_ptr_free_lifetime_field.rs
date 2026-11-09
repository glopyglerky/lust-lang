// expect: L0203
// Valid Rust: a struct field whose fn pointer type names the struct's
// lifetime. lust supports such types in fn signatures and `let`
// annotations only (W-BORROW3).
struct S<'a> { f: fn(&'a u8) -> u8 }
fn one(x: &u8) -> u8 { *x }
fn main() { let s = S { f: one }; println!("{}", (s.f)(&4)); }
