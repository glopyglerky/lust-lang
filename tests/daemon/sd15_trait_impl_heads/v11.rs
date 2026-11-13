use std::fmt::Debug;
trait Shape {
    type Out: Debug;
    fn area(&self, scale: u32) -> u32;
    fn name(&self) -> Self::Out;
    fn pair<T: Clone>(&self, x: T) -> (T, T) { (x.clone(), x) }
}
trait Label { fn label(&self) -> String; }
struct Sq(u32);
struct Rect<T> { w: T, h: T }
impl Shape for Sq {
    type Out = String;
    fn area(&self, scale: u32) -> u32 { self.0 * self.0 * scale }
    fn name(&self) -> Self::Out { format!("sq{}", self.0) }
}
impl<T: Copy + Into<u32>> Shape for Rect<T> {
    type Out = (u32, u32);
    fn area(&self, scale: u32) -> u32 where T: Default { self.w.into() * self.h.into() * scale }
    fn name(&self) -> Self::Out { (self.w.into(), self.h.into()) }
}
impl<S: Shape> Label for S { fn label(&self) -> String { format!("{:?}", self.name()) } }
fn total<S: Shape>(s: &S) -> u32 { s.area(2) }
fn dynsum(v: &[&dyn Label]) -> usize { v.iter().map(|x| x.label().len()).sum() }
fn direct(s: &Sq) -> u32 { s.area(1) + Shape::area(s, 3) + <Sq as Shape>::area(s, 4) }
fn main() {
    let s = Sq(3);
    let r = Rect { w: 2u8, h: 5u8 };
    println!("{} {} {} {:?} {} {:?}", total(&s), total(&r), direct(&s), r.name(), dynsum(&[&s, &r]), s.pair(1u8));
}
