// A builtin Clone, Ord, PartialOrd, or PartialEq method on a tuple, an
// array, or a reference, called through a type param, runs the builtin
// method. The crate's first written impl methods must not run instead.
use std::cmp::Ordering;
struct W;
impl W {
    fn first(&self) -> Ordering {
        Ordering::Equal
    }
    fn zero(&self) -> (u8, u8) {
        (9, 9)
    }
}
trait G<X> {
    fn g(&self, x: X) -> X;
}
struct Id;
impl<X: Copy> G<X> for Id {
    fn g(&self, x: X) -> X {
        x
    }
}
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct P(u8);
fn get<T: Clone>(x: &T) -> T {
    x.clone()
}
fn cmp<T: Ord>(a: &T, b: &T) -> Ordering {
    a.cmp(b)
}
fn pcmp<T: PartialOrd>(a: T, b: T) -> Option<Ordering> {
    a.partial_cmp(&b)
}
fn big<T: Ord>(a: T, b: T) -> T {
    a.max(b)
}
fn ne<T: PartialEq>(a: T, b: T) -> bool {
    a.ne(&b)
}
fn main() {
    println!("{:?} {:?} {:?}", get(&(1u8, 2u8)), get(&[P(3), P(4)]), get(&(P(5), 6u8)));
    println!("{:?} {:?}", cmp(&(1u8, 2u8), &(1u8, 3u8)), cmp(&[2u8], &[1u8]));
    println!("{:?} {:?} {:?}", pcmp((1, 2), (1, 3)), pcmp(&4, &3), pcmp("b", "a"));
    println!("{:?} {:?}", big((1, 2), (1, 3)), big(&4, &3));
    println!("{} {}", ne((1, 2), (1, 2)), ne([1], [2]));
    println!("{:?} {}", W.zero(), W.first() == Ordering::Equal);
    println!("{}", Id.g(get(&(7u8, 8u8))).1);
}
