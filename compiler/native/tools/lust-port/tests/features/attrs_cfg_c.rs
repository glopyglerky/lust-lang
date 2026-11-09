// feature: attrs.cfg
// form: the cold attribute on a helper fn
#[cold]
fn slow() -> i32 { 7 }
fn main() { println!("{}", slow()); }
