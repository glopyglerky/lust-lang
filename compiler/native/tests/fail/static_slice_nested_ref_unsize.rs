// expect: L0110
// A slice element `&&[1]` is no `&&[u8]`: the operand of `&` is no coercion site.
static A: &[&&[u8]] = &[&&[1]];
fn main() {
    println!("{:?}", A);
}
