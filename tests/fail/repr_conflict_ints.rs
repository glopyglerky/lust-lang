// expect: L0110
#[repr(u8, u16)]
enum E {
    A,
}

fn main() {
    let _e = E::A;
    println!("x");
}
