// Map and set lookups through written Borrow impls: a name type looked
// up by `&str` (its Hash hashes the text, as str does), a key looked up
// by its id (`Borrow<u32>`), and one by a byte slice.
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Name {
    text: String,
}

impl Name {
    fn new(s: &str) -> Name {
        Name { text: s.to_string() }
    }
    fn as_str(&self) -> &str {
        &self.text
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Borrow<str> for Name {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Id(u32);

impl Borrow<u32> for Id {
    fn borrow(&self) -> &u32 {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Bytes(Vec<u8>);

impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

fn main() {
    let mut m: HashMap<Name, u32> = HashMap::new();
    for (i, w) in ["alpha", "beta", "gamma"].iter().enumerate() {
        m.insert(Name::new(w), i as u32);
    }
    println!("{:?} {:?} {}", m.get("beta"), m.get("delta"), m.contains_key("gamma"));
    let key = String::from("alpha");
    if let Some(v) = m.get_mut(key.as_str()) {
        *v += 10;
    }
    println!("{:?} {:?}", m.remove("alpha"), m.len());
    println!("{:?}", m.get(&Name::new("gamma")));

    let mut s: HashSet<Name> = HashSet::new();
    s.insert(Name::new("x"));
    s.insert(Name::new("y"));
    println!("{} {} {:?}", s.contains("x"), s.contains("z"), s.get("y"));
    println!("{} {}", s.remove("x"), s.len());

    let ids: HashMap<Id, &str> = vec![(Id(4), "four"), (Id(9), "nine")].into_iter().collect();
    println!("{:?} {:?}", ids.get(&4), ids.get(&5));

    let mut bs: HashSet<Bytes> = HashSet::new();
    bs.insert(Bytes(vec![1, 2, 3]));
    let probe: &[u8] = &[1, 2, 3];
    println!("{} {}", bs.contains(probe), bs.contains(&[1u8, 2][..]));
}
