// ARCH 2.8: dynamic dispatch over two types, default and overridden methods.
trait Shape {
    fn area(&self) -> u32;
    fn name(&self) -> String { String::from("shape") }
}
struct Sq { s: u32 }
struct Rect { w: u32, h: u32 }
impl Shape for Sq {
    fn area(&self) -> u32 { self.s * self.s }
    fn name(&self) -> String { String::from("sq") }
}
impl Shape for Rect {
    fn area(&self) -> u32 { self.w * self.h }
}
fn show(s: &dyn Shape) {
    println!("{} {}", s.name(), s.area());
}
fn main() {
    let a = Sq { s: 3 };
    let b = Rect { w: 2, h: 5 };
    show(&a);
    show(&b);
    let v: &dyn Shape = &a;
    println!("{}", v.area());
}
