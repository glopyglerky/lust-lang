// Signed MIN literals are single literals, not a checked negate of the magnitude.
fn main() {
    println!("{} {} {} {}", -128i8, -32768i16, -2147483648i32, -9223372036854775808i64);
    println!("{}", -9223372036854775808isize);
    let a: i8 = -128;
    let b: i32 = -2147483648;
    let c: i64 = -9223372036854775808;
    println!("{} {} {}", a, b, c);
    println!("{} {}", -(128i8), -0x80i8);
    println!("{}", -128i8 + 1);
    match a {
        -128i8 => println!("min"),
        _ => println!("other"),
    }
    let x: i8 = -100;
    match x {
        -128i8..=-100i8 => println!("lo"),
        _ => println!("hi"),
    }
    println!("{}", min_i32());
}

fn min_i32() -> i32 {
    -2147483648
}
