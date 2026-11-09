// `borrow()` and `borrow_mut()` on a plain generic parameter under a
// `Borrow<Q>` bound, by method call and by trait path
// (`Borrow::borrow(x)`, `std::borrow::Borrow::borrow(x)`).
use std::borrow::{Borrow, BorrowMut};

fn by_method<B: Borrow<[u8]>>(b: &B) -> &[u8] {
    b.borrow()
}

fn by_path<B: Borrow<[u8]>>(b: &B) -> &[u8] {
    Borrow::borrow(b)
}

fn by_full_path<B: Borrow<str>>(b: &B) -> &str {
    std::borrow::Borrow::borrow(b)
}

fn by_q_of<B, Q: ?Sized>(b: &B) -> &Q
where
    B: Borrow<Q>,
{
    b.borrow()
}

fn mutate<B: BorrowMut<[u8]>>(b: &mut B) {
    b.borrow_mut()[0] = 9;
}

struct Wrap(String);

impl Borrow<str> for Wrap {
    fn borrow(&self) -> &str {
        &self.0
    }
}

fn main() {
    let v = vec![1u8, 2, 3];
    println!("{:?} {:?}", by_method(&v), by_path(&v));
    let a = [4u8, 5];
    println!("{:?}", by_method(&a));

    let s = String::from("hi");
    let x: &str = by_q_of(&s);
    println!("{} {}", x, by_full_path(&s));
    println!("{}", by_full_path(&Wrap(String::from("yo"))));

    let mut m = vec![1u8, 2];
    mutate(&mut m);
    println!("{:?}", m);

    let direct: &[u8] = <Vec<u8> as Borrow<[u8]>>::borrow(&v);
    println!("{:?}", direct);
}
