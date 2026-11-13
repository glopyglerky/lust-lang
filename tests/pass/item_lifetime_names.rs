// Lifetime names lust must keep accepting in const, static, alias, and
// impl item types (fx-ltdecl): `'static`, `'_`, the impl's own params,
// and names a `for<..>` binder declares.
static NAMES: &[&'static str] = &["a", "b"];
static ELIDED: Option<&'_ str> = Some("c");
const PICK: Option<for<'b> fn(&'b [u8]) -> &'b [u8]> = None;
type Show = fn(&'_ u8) -> String;
struct W<'a>(&'a u8);
impl<'a> W<'a> {
    const NONE: Option<&'a u8> = None;
    fn get(&self) -> u8 {
        Self::NONE.copied().unwrap_or(*self.0)
    }
}
fn first(x: &[u8]) -> &[u8] {
    &x[..1]
}
fn show(x: &u8) -> String {
    format!("<{x}>")
}
fn main() {
    let s: Show = show;
    println!("{:?} {:?} {:?} {}", NAMES, ELIDED, PICK.map_or(first(&[7, 8]), |f| f(&[1])), s(&W(&9).get()));
}
