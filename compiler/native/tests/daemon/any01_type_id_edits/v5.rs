// dyn Any type ids on one warm daemon: a downcast target behind an alias
// whose target changes (the fn bodies that test it do not), a field added
// to a tested type, a type renamed, and a supertrait upcast added.
use std::any::{Any, TypeId};
struct S(u8, u32);
struct R(u16);
type X = R;
trait Named: Any {
    fn name(&self) -> &'static str;
}
impl Named for S {
    fn name(&self) -> &'static str {
        "S"
    }
}
impl Named for R {
    fn name(&self) -> &'static str {
        "R"
    }
}
fn is_x(a: &dyn Any) -> bool {
    a.is::<X>()
}
fn tid() -> TypeId {
    TypeId::of::<X>()
}
fn main() {
    let xs: Vec<Box<dyn Named>> = vec![Box::new(S(1, 9)), Box::new(R(2))];
    for x in &xs {
        let a: &dyn Any = &**x;
        println!("{} {} {}", x.name(), is_x(a), (**x).type_id() == tid());
    }
}
