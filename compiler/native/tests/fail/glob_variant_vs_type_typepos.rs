// expect: L0507
// rustc E0659: the type position is ambiguous for the same reason.
enum E { B }
mod b {
    pub struct B;
}
use E::*;
use b::*;
fn f(_x: B) {}
fn main() {}
