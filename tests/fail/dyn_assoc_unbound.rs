// expect: L0702
// ARCH 2.8, rustc E0191: a dyn type must bind every associated type; lust has no assoc types in dyn yet.
trait Tr { type Out; fn get(&self) -> Self::Out; }
fn f(_x: &dyn Tr) {}
fn main() {}
