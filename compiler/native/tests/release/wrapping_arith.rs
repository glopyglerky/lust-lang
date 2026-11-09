// Release semantics (-C overflow-checks=off): + - * and unary - wrap for every integer width.
use std::hint::black_box as bb;

macro_rules! wrap_all {
    ($($t:ty),*) => {$(
        let (mx, mn): ($t, $t) = (bb(<$t>::MAX), bb(<$t>::MIN));
        let one: $t = bb(1);
        println!("{}: {} {} {} {}", stringify!($t), mx + one, mn - one, mx * bb(3 as $t), mx.wrapping_mul(mx) == mx * mx);
        let mut acc: $t = mx;
        acc += one;
        acc -= bb(2 as $t);
        acc *= bb(5 as $t);
        println!("  compound {}", acc);
    )*};
}

fn main() {
    wrap_all!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
    let (a, b, c, d): (i8, i32, i64, i128) = (bb(i8::MIN), bb(i32::MIN), bb(i64::MIN), bb(i128::MIN));
    println!("neg {} {} {} {}", -a, -b, -c, -d);
    let v: Vec<u8> = (0..1000u32).map(|x| (x % 251) as u8).collect();
    let s: u8 = v.iter().copied().sum();
    let p: i16 = (1..=30i16).product();
    let big: u64 = (0..10u64).map(|x| u64::MAX - x).sum();
    println!("sum {} product {} big {} count {}", s, p, big, v.iter().filter(|&&x| x > 100).count());
    let idx: usize = bb(usize::MAX);
    println!("index arith {}", idx + 2);
    let r: Vec<u8> = (250u8..=255).map(|x| x + 10).collect();
    println!("{:?}", r);
}
