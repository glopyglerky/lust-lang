// A where clause with an associated type binding is disproved by an impl whose
// binding differs: core's `impl Deref for &T` has `Target = T`, and Box<T>'s too,
// so neither &Local nor Box<Local> derefs to i32 (rustc accepts).
use std::ops::Deref;
struct Local;
trait Tr { fn f(&self) -> i32; }
impl<T: Deref<Target = i32>> Tr for T { fn f(&self) -> i32 { 1 } }
impl Tr for &Local { fn f(&self) -> i32 { 2 } }
impl Tr for Box<Local> { fn f(&self) -> i32 { 3 } }
fn main() {
    println!("{} {} {}", Box::new(7).f(), (&Local).f(), Box::new(Local).f());
}
