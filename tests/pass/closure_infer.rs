fn main() {
    let add = |a, b| a + b;
    println!("{}", add(2i64, 3));
    let describe = |x: u8| -> u16 { x as u16 + 100 };
    println!("{}", describe(5));
    let unit = || 42;
    println!("{}", unit());
    let with_block = |x: i64| {
        let y = x * 2;
        y + 1
    };
    println!("{}", with_block(10));
    let nested = |x: i64| {
        let inc = |y: i64| y + 1;
        inc(x) * 3
    };
    println!("{}", nested(4));
}
