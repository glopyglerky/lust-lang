// expect: L0632
// rustc E0379: functions in trait impls cannot be declared const.
trait T { fn t(&self) -> i32; }
struct S;
impl T for S { const fn t(&self) -> i32 { 3 } }
fn main() { println!("{}", S.t()); }
