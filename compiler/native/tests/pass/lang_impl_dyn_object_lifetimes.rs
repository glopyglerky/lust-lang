// Lang trait impls whose signatures hold dyn types with written object
// lifetimes (ts-rust frontend/module/cache.rs: a HashMap keyed through
// `Borrow<dyn ModuleKey + 'a>`). lust has no map lookup by a dyn key yet (L0640).
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
trait Key { fn parts(&self) -> (&str, u8); }
#[derive(PartialEq, Eq, Debug)]
struct K { a: String, b: u8 }
impl Key for K { fn parts(&self) -> (&str, u8) { (&self.a, self.b) } }
impl Key for (&str, u8) { fn parts(&self) -> (&str, u8) { *self } }
impl<'a> Borrow<dyn Key + 'a> for K {
    fn borrow(&self) -> &(dyn Key + 'a) { self }
}
impl Hash for K { fn hash<H: Hasher>(&self, s: &mut H) { self.parts().hash(s); } }
impl Hash for dyn Key + '_ { fn hash<H: Hasher>(&self, s: &mut H) { self.parts().hash(s); } }
impl PartialEq for dyn Key + '_ { fn eq(&self, o: &Self) -> bool { self.parts() == o.parts() } }
impl Eq for dyn Key + '_ {}
trait V { fn v(&self) -> u8; }
impl V for u8 { fn v(&self) -> u8 { *self } }
struct X(u8);
impl V for X { fn v(&self) -> u8 { self.0 } }
impl PartialEq for dyn V + '_ { fn eq(&self, o: &Self) -> bool { self.v() == o.v() } }
impl<'a> std::borrow::Borrow<dyn V + 'a> for X { fn borrow(&self) -> &(dyn V + 'static) { self } }
impl<'a> From<Box<dyn V>> for X { fn from(b: Box<dyn V + 'a>) -> X { X(b.v()) } }
fn conv(b: Box<dyn V>) -> u8 { X::from(b).0 }
fn main() {
    println!("{}", conv(Box::new(9u8)));
    let mut m: HashMap<K, i32> = HashMap::new();
    m.insert(K { a: "x".into(), b: 1 }, 7);
    let q: (&'static str, u8) = ("x", 1);
    let k: &dyn Key = &q;
    let key = K { a: "x".into(), b: 1 };
    let b: &dyn Key = <K as Borrow<dyn Key>>::borrow(&key);
    println!("{:?} {} {}", m.get(&key), k == b, k.parts().0);
    let a: &dyn V = &3u8;
    let b: &dyn V = &X(3);
    println!("{}", a == b);
}
