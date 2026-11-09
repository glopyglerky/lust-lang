// `{:x?}` and `{:X?}`: Debug that prints integers in hex, through
// compound values; signed values print their bits at the type width.
#[derive(Debug)]
struct Regs {
    a: u8,
    b: i16,
    c: Vec<u32>,
    d: (i8, u128),
}

fn main() {
    let r = Regs { a: 255, b: -1, c: vec![10, 4096], d: (-128, u128::MAX) };
    println!("{:x?}", r);
    println!("{:X?}", r);
    println!("{:#x?}", Some(171u8));
    println!("{:08x?} {:#06X?}", 255, 3054);
    println!("{:x?} {:X?}", 1.5f64, "str");
    println!("{:x} {:X} {:o} {:b}", -1i8, -2i16, -1i32, 5u8);
    println!("{:#X} {:#x} {:#o} {:#b} {:#010x} {:#6x} {:<#6x}|", 255, 255, 8, 5, 255, 255, 255);
    println!("{:#x} {:X}", i128::MIN, u64::MAX);
}
