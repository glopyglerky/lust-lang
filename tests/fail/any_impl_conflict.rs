// expect: L0615
// rustc E0119: core's blanket `impl<T: 'static + ?Sized> Any for T` covers every type (W-CHECK-G3).
use std::any::{Any, TypeId};
struct S;
impl Any for S { fn type_id(&self) -> TypeId { TypeId::of::<u8>() } }
fn main() { println!("{}", S.type_id() == TypeId::of::<u8>()); }
