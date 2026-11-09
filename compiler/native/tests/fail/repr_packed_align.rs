// expect: L0110
#[repr(packed, align(8))]
struct P {
    a: u8,
}

fn main() {
    let p = P { a: 1 };
    let a = p.a;
    println!("{}", a);
}
