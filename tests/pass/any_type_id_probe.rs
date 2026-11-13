// rustc's probe order for `type_id`: a reference step's pointee first, then
// the step under an autoref; `Box<dyn Any>` with Any in scope answers for the
// box, `*b` for the value; a dyn principal with Any as a supertrait answers
// through its vtable; `TypeId::of` takes unsized types (W-CHECK-G3).
use std::any::{Any, TypeId};
trait Shape: Any { fn area(&self) -> u32; }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 * self.0 } }
fn main() {
    let b: Box<dyn Any> = Box::new(3u8);
    println!("{}", b.type_id() == TypeId::of::<Box<dyn Any>>());
    println!("{}", (*b).type_id() == TypeId::of::<u8>());
    println!("{}", (&*b).type_id() == TypeId::of::<u8>());
    let r = &b;
    println!("{}", r.type_id() == TypeId::of::<Box<dyn Any>>());
    let s = "abc";
    println!("{}", s.type_id() == TypeId::of::<str>());
    println!("{}", TypeId::of::<[u8]>() == TypeId::of::<[u16]>());
    println!("{}", TypeId::of::<dyn Any>() == TypeId::of::<dyn Any + Send>());
    let sq = Sq(4);
    let d: &dyn Shape = &sq;
    println!("{} {}", d.area(), d.type_id() == TypeId::of::<Sq>());
    let v = vec![1u8];
    println!("{}", v.type_id() == TypeId::of::<Vec<u8>>());
    let t = (1u8, 'c');
    println!("{}", t.type_id() == TypeId::of::<(u8, char)>());
}
