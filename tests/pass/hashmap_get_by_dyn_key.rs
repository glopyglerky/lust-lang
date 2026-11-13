// A HashMap lookup through `Borrow<dyn Key + 'a>` (ts-rust frontend/module/cache.rs).
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
trait Key { fn id(&self) -> u8; }
#[derive(PartialEq, Eq)]
struct K(u8);
impl Key for K { fn id(&self) -> u8 { self.0 } }
impl Key for u8 { fn id(&self) -> u8 { *self } }
impl<'a> Borrow<dyn Key + 'a> for K { fn borrow(&self) -> &(dyn Key + 'a) { self } }
impl Hash for K { fn hash<H: Hasher>(&self, s: &mut H) { self.id().hash(s); } }
impl Hash for dyn Key + '_ { fn hash<H: Hasher>(&self, s: &mut H) { self.id().hash(s); } }
impl PartialEq for dyn Key + '_ { fn eq(&self, o: &Self) -> bool { self.id() == o.id() } }
impl Eq for dyn Key + '_ {}
fn main() {
    let mut m: HashMap<K, i32> = HashMap::new();
    m.insert(K(1), 7);
    println!("{:?}", m.get(&1u8 as &dyn Key));
}
