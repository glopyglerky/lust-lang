// expect: L0203
// rustc accepts; lust rejects named lifetimes in a closure's return
// annotation (L0203; ts-rust has none).
fn f<'a>(v: &'a String) -> &'a str {
    let first = |s: &'a String| -> &'a str { &s[..1] };
    first(v)
}
fn main() { let v = String::from("ab"); println!("{}", f(&v)); }
