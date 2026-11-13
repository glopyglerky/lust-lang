// Pieces fill the 1024-byte buffer one write at a time: the 1000-byte
// piece is held, the next piece does not fit, so the buffer is flushed
// and only the short tail is lost at the abort.
// panic: attempt to add with overflow
fn main() {
    let big = "z".repeat(1000);
    let small = "y".repeat(100);
    print!("{}{}", big, small);
    let n: u8 = bb(255);
    println!("{}", n + 1);
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
