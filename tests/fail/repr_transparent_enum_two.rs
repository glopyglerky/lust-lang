// expect: L0110
#[repr(transparent)]
enum E {
    A(u32),
    B,
}

fn main() {
    let _e = E::B;
    println!("x");
}
