// expect: L0312
// rustc E0506: a trait's default const value is borrow-checked even with no use.
trait Tr {
    const C: u32 = {
        let mut a = 1u32;
        let r = &a;
        a += 1;
        *r + a
    };
}
fn main() {
    println!("x");
}
