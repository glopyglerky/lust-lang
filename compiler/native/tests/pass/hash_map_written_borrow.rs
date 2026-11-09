// A lookup by str through a written Borrow impl (the derived Hash of
// K(String) hashes the String as a str, so the lookup finds the key).
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq, Debug)]
struct K(String);

impl Borrow<str> for K {
    fn borrow(&self) -> &str {
        &self.0
    }
}

fn main() {
    let mut m = HashMap::new();
    m.insert(K("a".to_string()), 1);
    println!("{:?} {:?}", m.get("a"), m.get("b"));
}
