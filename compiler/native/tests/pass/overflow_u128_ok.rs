// Near-bound u128 arithmetic that stays in range.
fn main() {
    let x: u128 = u128::MAX - 1;
    println!("{}", x + 1);
    let y: u128 = u128::MAX >> 127;
    println!("{}", y);
    let z: u128 = (1 << 127) + 1;
    println!("{}", z);
    let w: u128 = u128::MAX >> 64;
    println!("{}", w + 1);
}
