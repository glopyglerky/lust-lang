// ts-rust's AnyValue shape: the blanket impl's where clause holds for
// `Box<dyn AnyValue>` (the box is Marsh too), so `as_any` on the box
// picks the blanket at the box step; through `*b` it reaches the object.
use std::any::Any;
trait Marsh {
    fn m(&self) -> String;
}
impl Marsh for i32 {
    fn m(&self) -> String {
        format!("i{self}")
    }
}
impl<T: Marsh + ?Sized> Marsh for Box<T> {
    fn m(&self) -> String {
        format!("box({})", (**self).m())
    }
}
trait AnyValue: Any + Marsh {
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any + Marsh> AnyValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
fn main() {
    let b: Box<dyn AnyValue> = Box::new(5i32);
    println!("{:?}", (*b).as_any().downcast_ref::<i32>());
    println!("{}", b.as_any().is::<i32>());
    println!("{}", (*b).as_any().is::<i32>());
    println!("{} {}", b.m(), (*b).m());
}
