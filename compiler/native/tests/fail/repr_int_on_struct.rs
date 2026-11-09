// expect: L0110
#[repr(u8)]
struct T {
    a: u8,
}

fn main() {
    let _t = T { a: 1 };
}
