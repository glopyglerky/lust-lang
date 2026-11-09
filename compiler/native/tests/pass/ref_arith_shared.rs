// Arithmetic through shared references, as Rust allows for `&T` (but not `&mut T`).
fn main() {
    let a: i64 = 20;
    let b: i64 = 6;
    let r = &a;
    println!("{} {} {}", r + 1, 1 + r, &a + &b);
    println!("{} {} {} {}", &a - &b, &a * 2, &a / 3, &a % 7);
    println!("{} {} {} {}", &a & 12, &a | 1, &a ^ 5, &a << 1);
    println!("{} {}", &a >> &b, &b >> 1);
    println!("{} {}", -&a, !&b);
    let mut y: i64 = 1;
    y += &a;
    y -= &b;
    println!("{}", y);
    let t: bool = true;
    println!("{}", &t & true);
    let u: u8 = 5;
    println!("{}", &u + 1);
    println!("{} {}", &a < &b, &b < &a);
    let mut m: i64 = 20;
    let mut n: i64 = 6;
    println!("{} {}", &mut m == &mut n, &mut n < &mut m);
    println!("{}", &a == &mut m);
}
