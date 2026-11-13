// expect: L0110
// The pack bound must be a power of two (rustc E0589).
#[repr(packed(3))]
struct S {
    a: u8,
}

fn main() {
    let s = S { a: 1 };
    println!("{}", s.a);
}
