// expect: L0110
#[repr(packed)]
enum E {
    A(u8),
}

fn main() {
    let _e = E::A(1);
    println!("x");
}
