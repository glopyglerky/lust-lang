use std::cell::RefCell;
use std::rc::Rc;

/// An insertion-ordered map with a written Default impl.
pub struct Map<K, V> {
    pub keys: Vec<K>,
    pub vals: Vec<V>,
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Map { keys: Vec::new(), vals: Vec::new() }
    }
}

impl<K: PartialEq, V> Map<K, V> {
    pub fn insert(&mut self, k: K, v: V) {
        self.keys.push(k);
        self.vals.push(v);
    }
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

#[derive(Default, Debug)]
pub struct Counter {
    pub n: u64,
    pub tag: String,
}

/// An alias the dependency exports.
pub type Shared = Rc<RefCell<Map<String, u32>>>;
