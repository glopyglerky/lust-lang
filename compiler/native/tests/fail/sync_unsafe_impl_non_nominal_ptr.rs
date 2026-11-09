// expect: L0110
struct Foo;
unsafe impl Send for *const Foo {}
fn main() {
    println!("x");
}
