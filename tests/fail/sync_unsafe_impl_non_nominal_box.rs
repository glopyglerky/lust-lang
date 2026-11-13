// expect: L0110
struct Foo(*const u8);
unsafe impl Send for Box<Foo> {}
fn main() {
    println!("x");
}
