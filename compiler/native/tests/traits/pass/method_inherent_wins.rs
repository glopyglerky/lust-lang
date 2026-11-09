// ARCH 2.10 step 2: an inherent method in the same probe wins
// over a trait method. ARCH 2.10 step 5: a body that picked the
// trait method must recheck when the inherent method appears.
// From the sol probes method_change_after.rs.
struct S;
trait A {
    fn m(&self) -> u8;
}
impl A for S {
    fn m(&self) -> u8 {
        1
    }
}
impl S {
    fn m(&self) -> u8 {
        2
    }
}
fn main() {
    println!("{}", S.m());
}
