// expect: L0641
// Valid Rust: {:p} of a fat raw pointer prints its address; lust does not build it (tf-fatptr).
trait Tr {
    fn f(&self) -> u32;
}
struct S(u32);
impl Tr for S {
    fn f(&self) -> u32 {
        self.0
    }
}
fn main() {
    let p: *const str = "x";
    println!("{}", format!("{:p}", p).len() > 2);
}
