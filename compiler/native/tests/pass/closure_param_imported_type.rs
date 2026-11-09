// A closure parameter type names an imported std type (the resolver
// walks closure parameter types like other written types).
use std::borrow::Cow;
use std::collections::HashMap;

fn main() {
    let a: Cow<str> = Cow::Borrowed("abc");
    let f = |c: &Cow<str>| c.len() * 2;
    let g = |m: &HashMap<u8, u8>| -> usize { m.len() };
    let mut m = HashMap::new();
    m.insert(1u8, 2u8);
    println!("{} {}", f(&a), g(&m));
}
