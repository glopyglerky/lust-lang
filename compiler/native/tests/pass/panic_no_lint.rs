// Shapes the unconditional_panic and arithmetic_overflow lints do
// not reject: dead branches, slice and Vec indexes (a runtime
// length), a known-nonzero divisor, a dead enum arm, and allows.
enum E {
    A,
    B,
}

#[allow(unconditional_panic)]
fn allowed_panic() -> i32 {
    1 / 0
}

#[allow(arithmetic_overflow)]
fn allowed_overflow() -> u8 {
    let a: u8 = 255;
    a + 1
}

fn main() {
    let a = [1, 2, 3];
    if false {
        let _x = 1i32 / 0;
        let _y = a[9];
    }
    let s: &[i32] = &[10, 20, 30];
    let i = s.len() - 1;
    println!("{}", s[i]);
    let v = vec![4, 5, 6];
    println!("{}", v[2]);
    let z: i32 = 2;
    println!("{}", 10 / z);
    let big: u8 = 255;
    let e = E::B;
    match e {
        E::A => {
            let _b = big + 1;
            println!("a");
        }
        E::B => println!("b"),
    }
    let _ = allowed_panic;
    let _ = allowed_overflow;
}
