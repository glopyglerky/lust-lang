// rustc-error: E0793
// lust-expect: L0651
// ARCH 4.4: taking a reference to a packed field is an error.
// From the sol probe packed_reference.rs.
#[repr(packed)]
struct P {
    a: u8,
    b: u32,
}
fn main() {
    let p = P { a: 0, b: 1 };
    let x = &p.b;
    println!("{}", x);
}
