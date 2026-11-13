// ARCH 2.13: literals, arithmetic and comparisons on ints,
// bools, and chars; const fn calls whose bodies stay in the
// supported list (blocks, let, if, match, bounded loops, and
// calls to other such const fns).
const fn sum_to(n: u32) -> u32 {
    let mut total = 0;
    let mut i = 0;
    while i < n {
        total += i;
        i += 1;
    }
    total
}
const fn classify(n: u32) -> u8 {
    match n {
        0 => 0,
        1..=9 => 1,
        _ => 2,
    }
}
const A: u32 = sum_to(5);
const B: u32 = A * 2 + 1;
const FLAG: bool = B > 10 && A == 10;
const LETTER: char = 'x';
const NEXT: char = if FLAG { 'y' } else { 'z' };
static TABLE: [u32; 3] = [sum_to(2), sum_to(3), classify(42) as u32];
fn main() {
    println!("{A} {B} {FLAG} {LETTER} {NEXT} {} {} {}", TABLE[0], TABLE[1], TABLE[2]);
}
