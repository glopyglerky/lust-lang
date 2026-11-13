// expect: L0702
// Valid Rust (ARCH 2.8): `where Self: Send` on a method keeps the trait dyn compatible in rustc;
// lust does not check that bound at dyn calls yet.
trait T {
    fn f(&self) -> u8;
    fn g(&self) -> u8 where Self: Send { 9 }
}
struct S(u8);
impl T for S { fn f(&self) -> u8 { self.0 } }
fn main() { let s = S(3); let d: &(dyn T + Send) = &s; println!("{} {}", d.f(), d.g()); }
