// Integer types, casts, bit operations, wrapping math.
fn main() {
    let a: u8 = 200;
    let b = a as i8;
    let c = -1i32 as u32;
    let d = 300i32 as u8;
    let e = 'A' as u32 + 1;
    let f = 66u8 as char;
    println!("{} {} {} {} {}", b, c, d, e, f);
    let x: i64 = -17;
    println!("{} {} {}", x / 5, x % 5, x.abs());
    let bits: u32 = 0b1011;
    println!("{} {} {} {}", bits << 2, bits >> 1, bits & 6, bits | 4);
    println!("{} {}", 1i64.wrapping_add(2), 7u64.wrapping_sub(9));
    let m: i32 = 2147483647;
    println!("{}", m.wrapping_add(1));
    println!("{} {}", 10.min(3), (-4i16).max(-9));
    println!("{}", 3u64.pow(20));
    let n = std::env::args().count();
    println!("args {}", n);
    println!("{}", true as i32 + false as i32);
}
