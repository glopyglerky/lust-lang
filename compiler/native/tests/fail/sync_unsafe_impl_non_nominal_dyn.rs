// expect: L0110
trait Tr {}
struct Foo;
impl Tr for Foo {}
unsafe impl Send for dyn Tr {}
fn main() {
    println!("x");
}
