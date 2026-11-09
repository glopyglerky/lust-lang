// expect: L0203
// Both reject: rustc as the result borrows `w` (`'b`) not `'a`; lust as
// named lifetimes in closure return types are not supported yet.
fn f<'a, 'b>(v: &'a String, w: &'b String) -> &'a str {
    let first = |s: &'a String| -> &'a str { &s[..1] };
    first(w)
}
fn main() { let v = String::from("ab"); println!("{}", f(&v, &v)); }
