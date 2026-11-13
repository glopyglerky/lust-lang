// The keyed instance walk after an `impl Trait` body edit (w-sig-small
// round 6): the edit moves the environment hash, but when the kept
// program stands (same hidden type) only `Checked` keys can move, so the
// instance read sets (the closure bodies here) take the keyed answer. v2
// and v3 keep the hidden type u32; v4 changes it to &str (the full
// walk); v5 goes back; v6 makes `user` reject (E0277: the opaque type is
// not Copy); v7 reverts. The const item takes the resolver path.
#![allow(unused)]
const ROUTE: u8 = 0;
fn val() -> impl std::fmt::Debug {
    "s"
}
fn twice<T: std::fmt::Debug>(t: T) -> String {
    format!("{:?}{:?}", t, t)
}
fn id<T: Clone>(t: T) -> T {
    t.clone()
}
fn need_copy<T: Copy>(t: T) -> T {
    t
}
fn user() -> String {
    twice(val())
}
fn other() -> String {
    let c = |a: u8| a + 1;
    let d = |s: &str| twice(s);
    twice(id(c(5))) + &d("x")
}
fn main() {
    println!("{} {}", user(), other());
}
