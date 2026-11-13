// Comparison operators and methods through written PartialEq/PartialOrd
// impls: an overridden lt/ge is called, the rest follow core's default
// bodies (ne through eq, lt/le/gt/ge through partial_cmp), in rustc's call
// order. Also impls with another Rhs type, and generic impls.
use std::cmp::Ordering;

#[derive(Debug)]
struct X(u8);
impl PartialEq for X {
    fn eq(&self, o: &X) -> bool { println!("eq {} {}", self.0, o.0); self.0 == o.0 }
}
impl Eq for X {}
impl PartialOrd for X {
    fn partial_cmp(&self, o: &X) -> Option<Ordering> { println!("pcmp {} {}", self.0, o.0); self.0.partial_cmp(&o.0) }
    fn lt(&self, o: &X) -> bool { println!("lt {} {}", self.0, o.0); self.0 < o.0 }
    fn ge(&self, o: &X) -> bool { println!("ge {} {}", self.0, o.0); self.0 >= o.0 }
}
impl Ord for X {
    fn cmp(&self, o: &X) -> Ordering { println!("cmp {} {}", self.0, o.0); self.0.cmp(&o.0) }
}

fn overrides() {
    let (a, b) = (X(1), X(2));
    println!("{} {} {} {}", a < b, a <= b, a > b, a >= b);
    println!("{} {} {} {}", a.lt(&b), a.le(&b), a.gt(&b), a.ge(&b));
    println!("{} {}", a != b, a.ne(&b));
    println!("{} {}", &a < &b, (&a).lt(&b));
    println!("{:?} {:?}", X(5).max(X(4)), X(5).min(X(4)));
}

#[derive(Debug)]
struct P(u8);
impl PartialEq<u8> for P { fn eq(&self, o: &u8) -> bool { self.0 == *o } }
impl PartialEq<P> for u8 { fn eq(&self, o: &P) -> bool { *self == o.0 } }
impl PartialOrd<u8> for P { fn partial_cmp(&self, o: &u8) -> Option<Ordering> { self.0.partial_cmp(o) } }

fn rhs_types() {
    println!("{} {} {} {}", P(1) == 1u8, P(1) != 2u8, 3u8 == P(3), 3u8 != P(3));
    println!("{} {} {}", P(1) < 2u8, P(5) >= 5u8, P(1).gt(&0u8));
}

struct NeP(u8);
impl PartialEq for NeP {
    fn eq(&self, o: &NeP) -> bool { println!("eq"); self.0 == o.0 }
    fn ne(&self, o: &NeP) -> bool { println!("ne"); self.0 != o.0 }
}

fn ne_override() {
    println!("{} {} {}", NeP(1) != NeP(2), NeP(1).ne(&NeP(1)), NeP(3) == NeP(3));
}

#[derive(Debug)]
struct W<T>(T, u8);
impl<T: PartialEq> PartialEq for W<T> { fn eq(&self, o: &W<T>) -> bool { println!("W eq"); self.0 == o.0 } }
impl<T: PartialOrd> PartialOrd for W<T> {
    fn partial_cmp(&self, o: &W<T>) -> Option<Ordering> { println!("W pcmp"); self.0.partial_cmp(&o.0) }
}
impl<T: Eq> Eq for W<T> {}
impl<T: Ord> Ord for W<T> { fn cmp(&self, o: &W<T>) -> Ordering { println!("W cmp"); self.0.cmp(&o.0) } }
fn gne<T: PartialEq>(a: T, b: T) -> bool { a != b }
fn gge<T: PartialOrd>(a: &T, b: &T) -> bool { a >= b }

fn generic_impls() {
    println!("{} {}", W(1, 0) == W(1, 9), W(1, 0) != W(2, 0));
    println!("{} {} {}", W(1.0, 0) < W(f64::NAN, 0), W(2, 0) >= W(1, 0), W(String::from("a"), 0).le(&W(String::from("b"), 0)));
    println!("{} {}", gne(W('a', 0), W('a', 1)), gge(&W(3u64, 0), &W(4u64, 0)));
    println!("{:?} {:?}", W(1, 1).max(W(1, 2)), W(5, 1).clamp(W(1, 0), W(3, 0)));
    let (mut a, mut b) = (W(1, 0), W(2, 0));
    println!("{} {}", &mut a == &mut b, &a != &b);
}

fn main() {
    overrides();
    rhs_types();
    ne_override();
    generic_impls();
}
