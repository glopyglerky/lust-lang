// With Borrow in scope, `p.borrow()` on a type that derefs to a RefCell
// picks core's reflexive `Borrow<P> for P` at the first autoderef step,
// as rustc does, not RefCell's inherent `borrow` behind the Deref.
#![allow(unused_variables)]
use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::Deref;

#[derive(Debug)]
struct P(RefCell<u8>);

impl Deref for P {
    type Target = RefCell<u8>;
    fn deref(&self) -> &RefCell<u8> {
        &self.0
    }
}

fn via<K: Borrow<Q>, Q: ?Sized + std::fmt::Debug>(k: &K) -> String {
    format!("{:?}", k.borrow())
}

fn main() {
    let p = P(RefCell::new(5));
    let r = p.borrow();
    println!("{:?}", r);
    println!("{}", via::<P, P>(&p));
    println!("{}", *p.0.borrow());
}
