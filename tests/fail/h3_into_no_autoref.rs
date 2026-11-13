// expect: L0104
// rustc E0277: `into` resolves at the by-value step (core's blanket Into may hold
// for S), so the From<&S> impl is never reached through an autoref.
struct S;
impl<'a> From<&'a S> for String { fn from(_: &'a S) -> String { "s".into() } }
fn main() {
    let s = S;
    let t: String = s.into();
    println!("{}", t);
}
