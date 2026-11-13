// Ord::max/min/clamp by core's default bodies: compare with PartialOrd::lt
// (le and gt for clamp, after `assert!(min <= max)`), return the winner,
// and drop the other values inside the call in reverse parameter order.
// Through written impls, derived impls, and core types.
use std::cmp::Ordering;

#[derive(Debug)]
struct X(u8, &'static str);
impl Drop for X { fn drop(&mut self) { println!("drop {}{}", self.1, self.0); } }
impl PartialEq for X { fn eq(&self, o: &X) -> bool { println!("eq"); self.0 == o.0 } }
impl Eq for X {}
impl PartialOrd for X {
    fn partial_cmp(&self, o: &X) -> Option<Ordering> { println!("pcmp {} {}", self.0, o.0); self.0.partial_cmp(&o.0) }
}
impl Ord for X { fn cmp(&self, o: &X) -> Ordering { println!("cmp {} {}", self.0, o.0); self.0.cmp(&o.0) } }

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct S(u8, &'static str);
impl Drop for S { fn drop(&mut self) { println!("drop {}", self.1); } }

#[derive(PartialEq, Eq, Ord, Debug)]
struct D(u8);
impl PartialOrd for D { fn partial_cmp(&self, o: &D) -> Option<Ordering> { println!("D pcmp"); Some(self.cmp(o)) } }

fn written() {
    println!("-- max"); let m = X(1, "a").max(X(2, "b")); println!("got {:?}", m);
    println!("-- max eq"); let m2 = X(3, "a").max(X(3, "b")); println!("got {:?}", m2);
    println!("-- min"); let n = X(1, "a").min(X(2, "b")); println!("got {:?}", n);
    println!("-- min eq"); let n2 = X(3, "a").min(X(3, "b")); println!("got {:?}", n2);
    println!("-- clamp lo"); let c = X(1, "s").clamp(X(2, "lo"), X(4, "hi")); println!("got {:?}", c);
    println!("-- clamp hi"); let c2 = X(9, "s").clamp(X(2, "lo"), X(4, "hi")); println!("got {:?}", c2);
    println!("-- clamp mid"); let c3 = X(3, "s").clamp(X(2, "lo"), X(4, "hi")); println!("got {:?}", c3);
    println!("-- end");
}

fn derived() {
    let m = S(1, "a").max(S(2, "b"));
    println!("got {:?}", m);
    let c = S(5, "s").clamp(S(1, "lo"), S(3, "hi"));
    println!("got {:?}", c);
    println!("{:?} {:?}", 'z'.max('a'), "x".to_string().min("y".to_string()));
    println!("{} {:?}", 7.clamp(1, 5), (1, 2).max((1, 3)));
    println!("{} {:?} {:?}", D(1) < D(2), D(1).max(D(2)), D(3).cmp(&D(1)));
}

fn main() {
    written();
    derived();
}
