// Known-value shapes the arithmetic_overflow lint does not reject:
// a value stored back into a local whose address is taken later
// (every value below is printed), an operand behind a reference, a
// value assigned inside a branch, and an allowed overflow.
#[allow(arithmetic_overflow)]
fn allowed() -> u8 {
    let a: u8 = 254;
    a + 1
}

fn g() -> bool {
    true
}

fn main() {
    let mut a: u8 = 250;
    a = a + 3;
    println!("{}", a);
    let b: u8 = 200;
    let r = &b;
    println!("{}", *r + 50);
    let mut c: u8 = 0;
    if g() {
        c = 250;
    }
    println!("{}", c + 3);
    let mut d: u8 = 250;
    d += 3;
    println!("{}", d);
    println!("{}", allowed());
    println!("{}", 0xFFu8 << 4);
}
