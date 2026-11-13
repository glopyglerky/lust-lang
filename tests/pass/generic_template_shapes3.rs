// Template checks over projections with where-clause bounds, supertrait
// methods in default bodies, bounded inherent impls, and generic trait
// methods called with a turbofish.
trait A { fn a(&self) -> u8; }
trait B: A { fn b(&self) -> u8 { self.a() + 1 } }
struct S(u8);
impl A for S { fn a(&self) -> u8 { self.0 } }
impl B for S {}
fn use_b<T: B>(t: &T) -> u8 { t.a() + t.b() }
trait Shape { fn area(&self) -> u32; }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 * self.0 } }
struct W<T> { x: T }
impl<T: Shape> W<T> { fn area(&self) -> u32 { self.x.area() } }
fn first_clone<I: Iterator>(mut it: I) -> Option< <I as Iterator>::Item> where <I as Iterator>::Item: Clone {
    match it.next() { Some(x) => Some(x.clone()), None => None }
}
struct Up(u32);
impl Iterator for Up { type Item = u32; fn next(&mut self) -> Option<u32> { if self.0 >= 3 { None } else { self.0 += 1; Some(self.0) } } }
trait Make { fn make<U: Default>(&self) -> U; }
impl Make for S { fn make<U: Default>(&self) -> U { U::default() } }
fn via<T: Make>(t: &T) -> u16 { t.make::<u16>() + 1 }
fn pair<T: Clone, U: Copy>(t: &T, u: U) -> (T, U, U) { (t.clone(), u, u) }
fn main() {
    println!("{}", use_b(&S(3)));
    println!("{}", W { x: Sq(4) }.area());
    println!("{:?}", first_clone(Up(0)));
    println!("{}", via(&S(1)));
    let p = pair(&String::from("a"), 2u8);
    println!("{} {} {}", p.0, p.1, p.2);
}
