// Stores into containers relate by the element type's variance: covariant
// parts may be longer, a `&mut dyn` element coerces where rustc has a
// coercion site, and a higher-ranked fn pointer pushed directly coerces to
// a free one.
use std::cell::Cell;
use std::collections::HashMap;
trait P {
    fn id(&self) -> u32;
}
struct S(u32);
impl P for S {
    fn id(&self) -> u32 {
        self.0
    }
}
fn push_cov<'b, 'c: 'b>(v: &mut Vec<&'b u8>, x: &'c u8) {
    v.push(x);
    v.extend([x]);
}
fn push_inv<'b>(v: &mut Vec<Cell<&'b u8>>, x: Cell<&'b u8>) {
    v.insert(0, x);
}
fn put_dyn<'a: 'x, 'x>(c: &'a mut (dyn P + 'a), v: &mut Option<&'x mut (dyn P + 'x)>) {
    let g = c;
    *v = Some(g);
}
fn push_hr<'b>(v: &mut Vec<&'b fn(&'b u8) -> u8>, x: &'b fn(&u8) -> u8) {
    v.push(x);
}
fn two(x: &u8) -> u8 {
    *x * 2
}
fn main() {
    let a = 3u8;
    let mut v = Vec::new();
    push_cov(&mut v, &a);
    let mut w = Vec::new();
    push_inv(&mut w, Cell::new(&a));
    let mut s = S(7);
    let mut d: Option<&mut dyn P> = None;
    put_dyn(&mut s, &mut d);
    let f: fn(&u8) -> u8 = two;
    let mut fs = Vec::new();
    push_hr(&mut fs, &f);
    let mut m: HashMap<u8, Cell<&u8>> = HashMap::new();
    m.insert(1, Cell::new(&a));
    println!("{} {} {} {} {}", v.len(), w[0].get(), d.unwrap().id(), fs[0](&a), m[&1].get());
}
