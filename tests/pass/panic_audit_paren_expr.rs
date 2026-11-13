// A parenthesized expression spans its parens (rustc's HIR lowering):
// overflow in `(a + b)` reports at the `(`.
// panic: attempt to multiply with overflow
fn main() {
    let n = std::env::args().count() as u8;
    let a = (n + 1) * 2;
    println!("{}", a);
    let b = 3 * (n * 100);
    println!("{}", b);
}
