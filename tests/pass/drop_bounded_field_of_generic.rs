// A generic struct whose field type has a Drop impl with a trait bound
// (W<T> inside G<T>): lust instantiated W's drop for G's unbound param
// and rejected (L0612 no impl of Write for T0). Both Drop impls run, in
// rustc's order: G's body first, then the field's.
use std::io::Write;

struct W<T: Write> {
    obj: Option<T>,
}

impl<T: Write> W<T> {
    fn fin(&mut self, tag: &[u8]) -> std::io::Result<()> {
        self.obj.as_mut().unwrap().write_all(tag)?;
        Ok(())
    }
}

impl<T: Write> Drop for W<T> {
    fn drop(&mut self) {
        if self.obj.is_some() {
            let _ = self.fin(b"w");
        }
    }
}

struct G<T: Write> {
    inner: W<T>,
    k: u8,
}

impl<T: Write> Drop for G<T> {
    fn drop(&mut self) {
        self.k += 1;
        let _ = self.inner.fin(b"g");
    }
}

enum E<T: Write> {
    One(W<T>),
    Two(G<T>, u8),
}

fn make<T: Write>(t: T, two: bool) -> E<T> {
    if two { E::Two(G { inner: W { obj: Some(t) }, k: 0 }, 1) } else { E::One(W { obj: Some(t) }) }
}

fn main() {
    let mut v = Vec::new();
    {
        let _g = G { inner: W { obj: Some(&mut v) }, k: 0 };
    }
    println!("{:?}", String::from_utf8_lossy(&v));
    let mut a = Vec::new();
    let mut b = Vec::new();
    drop(make(&mut a, false));
    drop(make(&mut b, true));
    println!("{:?} {:?}", String::from_utf8_lossy(&a), String::from_utf8_lossy(&b));
}
