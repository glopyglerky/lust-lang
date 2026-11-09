// ARCH 2.10 step 2: with no inherent method, the trait method is
// picked. Pair with method_inherent_wins.rs: adding an inherent
// method with this name changes the answer to 2.
// From the sol probe method_change_before.rs.
struct S;
trait A {
    fn m(&self) -> u8;
}
impl A for S {
    fn m(&self) -> u8 {
        1
    }
}
fn main() {
    println!("{}", S.m());
}
