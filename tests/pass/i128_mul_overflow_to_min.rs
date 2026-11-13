// panic: attempt to multiply with overflow
// The product +2^127 wraps to i128::MIN; rustc panics. -2^127 is fine.
fn main() {
    let a: i128 = bb(-85070591730234615865843651857942052864); // -2^126
    println!("{}", a * bb(2i128));
    println!("{}", bb(1i128 << 64) * bb(-(1i128 << 63)));
    println!("{}", a * bb(-2i128));
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
