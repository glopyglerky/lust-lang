// macro: effect_cached
// source: crates/ts_goport/src/effect/typeparser/mod.rs:193
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::cell::RefCell;
use std::collections::HashMap;

// A field map stand-in: the macro calls .get(&key) and .insert(key, value)
// through &$tp.links(), so the map needs interior mutability.
#[derive(Default)]
pub struct Cache(RefCell<HashMap<u32, String>>);
impl Cache {
    pub fn get(&self, k: &u32) -> Option<String> { self.0.borrow().get(k).cloned() }
    pub fn insert(&self, k: u32, v: String) { self.0.borrow_mut().insert(k, v); }
}
#[derive(Default)]
pub struct Links { pub by_node: Cache, pub by_name: Cache }
pub struct TypeParser { links_store: Links, pub computes: RefCell<u32> }
impl TypeParser {
    pub fn links(&self) -> &Links { &self.links_store }
    pub fn label(&self, node: u32) -> String {
        cached!(self, by_node, node, {
            *self.computes.borrow_mut() += 1;
            format!("node-{node}")
        })
    }
    pub fn titled(&self, node: u32) -> String {
        // The labeled-block compute expression ts-rust passes.
        cached!(self, by_name, node, 'compute: {
            *self.computes.borrow_mut() += 1;
            break 'compute format!("title-{node}");
        })
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! effect_cached {
    ($tp:expr, $field:ident, $key:expr, $compute:expr) => {{
        let key = $key;
        if let Some(value) = $tp.links().$field.get(&key) {
            value.clone()
        } else {
            let value = $compute;
            $tp.links().$field.insert(key, value.clone());
            value
        }
    }};
}
pub use crate::effect_cached as cached;

fn main() {
    let tp = TypeParser { links_store: Links::default(), computes: RefCell::new(0) };
    println!("{} {} {}", tp.label(4), tp.label(4), tp.label(5));
    println!("{} computes={}", tp.titled(4), tp.computes.borrow());
}
