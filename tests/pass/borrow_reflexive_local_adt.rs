// core's reflexive `impl<T: ?Sized> Borrow<T> for T` holds for a user
// type: `p.borrow()` directly, and `k.borrow()` in a `K: Borrow<Q>`
// body instantiated at K = Q = a user type (check passed, but the
// instance failed in build before).
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Debug)]
struct P(String);

#[derive(Debug)]
struct G<T>(T);

fn same<K: Borrow<Q>, Q: ?Sized + PartialEq>(k: &K, q: &Q) -> bool {
    k.borrow() == q
}

fn first<K: Borrow<Q>, Q: ?Sized + std::fmt::Debug>(ks: &[K]) -> String {
    format!("{:?}", ks[0].borrow())
}

fn bump<T: BorrowMut<G<u8>>>(mut t: T) -> u8 {
    t.borrow_mut().0 += 1;
    t.borrow_mut().0
}

fn main() {
    let p = P("a".into());
    let q: &P = p.borrow();
    println!("{:?} {}", q, same(&p, &p));
    println!("{}", same(&String::from("x"), &String::from("x")));
    println!("{}", first::<P, P>(&[P("z".into())]));
    let mut g = G(3u8);
    let gm: &mut G<u8> = g.borrow_mut();
    gm.0 += 1;
    println!("{:?} {}", g, bump(G(9u8)));
    let mut m: HashMap<P, u8> = HashMap::new();
    m.insert(P("k".into()), 1);
    println!("{:?}", m.get(&P("k".into())));
}
