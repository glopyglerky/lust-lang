// expect: L0305
// rustc E0507: an expression statement moves its value, and a static cannot move out.
struct D(u8);
static S: D = D(1);
fn main() {
    S;
    println!("x");
}
