// Derives over `OnceCell` fields (W-BENCH2 bug 4): std's generic
// `Clone`, `Default`, `PartialEq`, and `Debug` impls for
// `OnceCell<T>` serve the derive glue, in a struct, an enum, a
// tuple, and a `Vec`.
use std::cell::OnceCell;
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct W {
    c: OnceCell<String>,
    cs: Vec<OnceCell<u32>>,
    t: (OnceCell<String>, u8),
}
#[derive(Clone, PartialEq, Debug)]
enum E {
    A(OnceCell<String>),
    B,
}
fn main() {
    let mut a = W::default();
    let _ = a.c.set("x".to_string());
    a.cs.push(OnceCell::new());
    let _ = a.cs[0].set(4);
    let b = a.clone();
    println!("{} {:?}", a == b, b);
    let e = E::A(OnceCell::new());
    if let E::A(c) = &e {
        let _ = c.set("y".to_string());
    }
    let f = e.clone();
    println!("{} {:?}", e == f, f);
}
