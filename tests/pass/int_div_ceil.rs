// Unsigned `div_ceil` (W-BENCH4 std gaps). Rust has no checked form.
fn main() {
    println!("{}", 7usize.div_ceil(2));
    println!("{}", 8usize.div_ceil(2));
    println!("{}", 0u32.div_ceil(5));
    println!("{}", u8::MAX.div_ceil(2));
    println!("{}", u64::MAX.div_ceil(1));
    println!("{}", 1u16.div_ceil(3));
    const C: usize = 10usize.div_ceil(3);
    println!("{C}");
    let a = 100u32;
    println!("{}", a.div_ceil(a - 97));
}
