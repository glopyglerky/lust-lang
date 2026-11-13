// An impl for &S / &mut S with a by-value self is found at the autoref step,
// before an inherent &mut self method of S (rustc's probe: S, &S, &mut S).
use std::ops::Add;
trait Tr { fn m(self) -> &'static str; }
struct S;
impl S { fn m(&mut self) -> &'static str { "inherent &mut self" } }
impl Tr for &S { fn m(self) -> &'static str { "trait for &S" } }
trait Tm { fn k(self) -> i32; }
struct K(i32);
impl Tm for &mut K { fn k(self) -> i32 { self.0 += 1; self.0 } }
#[derive(Debug)]
struct V(i32);
impl Add for &V { type Output = V; fn add(self, o: &V) -> V { V(self.0 + o.0) } }
struct C(Vec<i32>);
impl<'a> IntoIterator for &'a C { type Item = &'a i32; type IntoIter = std::slice::Iter<'a, i32>; fn into_iter(self) -> Self::IntoIter { self.0.iter() } }
fn main() {
    let mut s = S;
    println!("{}", s.m());
    println!("{}", Box::new(S).m());
    let mut k = K(1);
    println!("{}", k.k());
    let (a, b) = (V(1), V(2));
    println!("{:?}", a.add(&b));
    let c = C(vec![1, 2, 3]);
    println!("{}", c.into_iter().count());
}
