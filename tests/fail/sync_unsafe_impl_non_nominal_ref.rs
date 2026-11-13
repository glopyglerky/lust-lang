// expect: L0110
struct Foo(*const u8);
unsafe impl Send for &'static Foo {}
fn main() {
    println!("x");
}
