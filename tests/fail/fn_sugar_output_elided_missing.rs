// expect: L0202
// rustc E0106: two input lifetimes, so elision cannot pick the output's (also with the same name twice).
fn app<F: Fn(&u8, &u8) -> &u8>(f: F) {}
fn app2<'a, G: Fn(&'a u8, &'a u8) -> &u8>(g: G) {}

fn main() {}
