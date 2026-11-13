// join and concat over std and user Borrow<str> elements, as core's
// join_generic_copy: exact capacity; a written borrow is called once for the
// first part and twice for each other part; generic and direct calls.
use std::borrow::{Borrow, Cow};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
struct P(&'static str, Cell<u32>);
impl Borrow<str> for P {
    fn borrow(&self) -> &str {
        self.1.set(self.1.get() + 1);
        println!("borrow {} #{}", self.0, self.1.get());
        self.0
    }
}
struct W<T>(T, &'static str);
impl<T: std::fmt::Debug> Borrow<str> for W<T> {
    fn borrow(&self) -> &str {
        println!("borrow {:?}", self.0);
        self.1
    }
}
#[derive(Clone)]
struct N(String);
impl Borrow<str> for N {
    fn borrow(&self) -> &str {
        &self.0
    }
}
fn joined<S: Borrow<str>>(parts: &[S], sep: &str) -> String {
    parts.join(sep)
}
fn cat<S: Borrow<str>>(parts: &[S]) -> String {
    parts.concat()
}
fn main() {
    let b: Vec<Box<str>> = vec!["a".into(), "bc".into()];
    let s = joined(&b, ", ");
    println!("{s} {}", s.capacity());
    let c: Vec<Cow<str>> = vec![Cow::Borrowed("x"), Cow::Owned(String::from("yz"))];
    println!("{} {}", joined(&c, "-"), cat(&c));
    let r: Vec<Rc<str>> = vec![Rc::from("r1"), Rc::from("r2")];
    let a: [Arc<str>; 2] = [Arc::from("a1"), Arc::from("a2")];
    println!("{} {}", cat(&r), joined(&a, "+"));
    let p = [P("p", Cell::new(0)), P("qq", Cell::new(0)), P("r", Cell::new(0))];
    let s = joined(&p, "::");
    println!("{s} {}", s.capacity());
    println!("{}", cat(&p[..0]).capacity());
    println!("{}", cat(&p[..1]));
    let e: Vec<Box<str>> = Vec::new();
    println!("[{}]", joined(&e, "x"));
    user_generic();
}

fn user_generic() {
    let w = vec![W(1u8, "one"), W(2, "two")];
    println!("{}", joined(&w, " / "));
    println!("{}", w.concat());
    let arr = [N("a".into()), N("bb".into()), N(String::new()), N("c".into())];
    let s = arr.join("");
    println!("{s} {}", s.capacity());
    println!("{}", arr[1..].join("--"));
    let v: Vec<N> = arr.to_vec();
    println!("{}", joined(&v, ","));
    let e: [N; 0] = [];
    println!("[{}] {}", e.join("x"), e.concat().capacity());
}
