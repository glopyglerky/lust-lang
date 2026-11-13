// dyn Any type ids on one warm daemon: a downcast target behind an alias
// whose target changes (the fn bodies that test it do not), a field added
// to a tested type, a type renamed, and a supertrait upcast added.
use std::any::{Any, TypeId};
struct S(u8, u32);
struct T(u16);
type X = T;
fn is_x(a: &dyn Any) -> bool {
    a.is::<X>()
}
fn tid() -> TypeId {
    TypeId::of::<X>()
}
fn main() {
    let xs: Vec<Box<dyn Any>> = vec![Box::new(S(1, 9)), Box::new(T(2)), Box::new(3u8)];
    for x in &xs {
        println!("{} {}", is_x(&**x), (**x).type_id() == tid());
    }
}
