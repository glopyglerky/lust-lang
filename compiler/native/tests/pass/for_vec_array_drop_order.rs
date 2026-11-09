// A for loop over a Vec, array, boxed slice, map or Option value made by
// its head: on break, the rest of the elements drop before the head's
// temporaries, as in rustc (was a wrong output: the head's first).
use std::collections::HashMap;
struct T(&'static str);
impl Drop for T { fn drop(&mut self) { println!("drop {}", self.0); } }
fn mk(_t: &T) -> Vec<T> { vec![T("e1"), T("e2")] }
fn mka(_t: &T) -> [T; 2] { [T("a1"), T("a2")] }
fn mkb(_t: &T) -> Box<[T]> { vec![T("b1"), T("b2")].into_boxed_slice() }
fn mkh(_t: &T) -> HashMap<u8, T> { let mut m = HashMap::new(); m.insert(1, T("h1")); m }
fn mko(_t: &T) -> Option<T> { Some(T("o1")) }
fn main() {
    for x in mk(&T("hv")) { println!("got {}", x.0); break; }
    for x in mka(&T("ha")) { println!("got {}", x.0); break; }
    for x in mkb(&T("hb")) { println!("got {}", x.0); break; }
    for (_, x) in mkh(&T("hh")) { println!("got {}", x.0); }
    for x in mko(&T("ho")) { println!("got {}", x.0); }
    for x in mk(&T("hv2")).into_iter() { println!("got {}", x.0); break; }
    for x in mk(&T("hv3")).iter() { println!("got {}", x.0); break; }
}
