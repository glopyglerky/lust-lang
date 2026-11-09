// expect: L0640
// Valid Rust: a lookup through a generic written Borrow impl. The hash
// glue calls only non-generic Borrow impls on non-generic types.
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash)]
struct Tag<T>(T);

impl<T> Borrow<T> for Tag<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

fn main() {
    let mut m = HashMap::new();
    m.insert(Tag(3u8), "three");
    println!("{:?}", m.get(&3u8));
}
