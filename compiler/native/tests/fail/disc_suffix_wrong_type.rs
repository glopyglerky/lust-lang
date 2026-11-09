// expect: L0110
#[repr(u8)]
enum E {
    A = 1u16,
}

fn main() {
    let _e = E::A;
    println!("x");
}
