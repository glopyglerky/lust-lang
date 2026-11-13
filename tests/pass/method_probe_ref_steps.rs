// rustc's method probe walks the receiver's autoderef steps and, at each,
// tries by value, then `&`, then `&mut`. Core's impls for references
// answer at the first step that fits: `(&&s).clone()` is `&String`,
// `(&mut s).deref()` is core's `&mut T` impl, `(&mut &s).to_owned()` is
// `String`, and a call through a user `Deref` starts at the target place.
#![allow(noop_method_call, suspicious_double_ref_op)]
use std::borrow::Borrow;
use std::ops::Deref;
trait K {
    fn k(&self) -> &'static str;
}
impl K for String {
    fn k(&self) -> &'static str {
        "String"
    }
}
impl K for &String {
    fn k(&self) -> &'static str {
        "&String"
    }
}
impl K for &&String {
    fn k(&self) -> &'static str {
        "&&String"
    }
}
impl K for &mut String {
    fn k(&self) -> &'static str {
        "&mut String"
    }
}
impl K for &&mut String {
    fn k(&self) -> &'static str {
        "&&mut String"
    }
}
impl K for str {
    fn k(&self) -> &'static str {
        "str"
    }
}
impl K for &str {
    fn k(&self) -> &'static str {
        "&str"
    }
}
fn show<T: K + ?Sized>(t: &T) -> &'static str {
    t.k()
}
struct W<T>(T);
impl<T> Deref for W<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
#[derive(Clone, Debug)]
struct S(i32);
fn twice<'a, T>(t: &&'a T) -> &'a T {
    t.clone()
}
fn main() {
    let s = String::from("s");
    let rr = &&s;
    println!("{}", show(&rr.clone()));
    let rrr = &rr;
    println!("{}", show(&rrr.clone()));
    let mut m = String::from("m");
    let rm = &&mut m;
    println!("{}", show(&rm.clone()));
    let mut r = &s;
    let mr = &mut r;
    println!("{}", show(&mr.clone()));
    println!("{}", show(&mr.to_owned()));
    println!("{}", show(&rr.to_owned()));
    let w = W(&s);
    println!("{}", show(&w.clone()));
    println!("{}", show(&w.to_owned()));
    let ws = W("lit");
    println!("{}", show(&ws.to_owned()));
    let mut t = String::from("t");
    let mt = &mut t;
    println!("{}", show(&mt.deref()));
    let b: &String = mt.borrow();
    println!("{}", show(&b));
    let sr = &S(4);
    let o = (&sr).to_owned();
    println!("{:?} {:?}", o, S(5).to_owned());
    println!("{}", show(&twice(&&s)));
    let mut z = &s;
    let u = String::from("u");
    z.clone_from(&&u);
    println!("{} {}", z, s);
}
