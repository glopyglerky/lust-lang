// expect: L0651
#[repr(packed)]
struct P {
    a: u8,
    b: u64,
}

fn main() {
    let p = P { a: 1, b: 2 };
    let r = &p.b;
    println!("{}", r);
}
