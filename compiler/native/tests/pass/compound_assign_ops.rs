// Bit and shift compound assignments.
fn main() {
    let mut x: i64 = 6;
    x &= 3;
    x |= 8;
    x ^= 1;
    x <<= 2;
    x >>= 1;
    println!("{}", x);
    let mut b: bool = true;
    b &= false;
    b |= true;
    println!("{}", b);
    let mut u: u32 = 1;
    u <<= 10;
    u >>= 4;
    println!("{}", u);
}
