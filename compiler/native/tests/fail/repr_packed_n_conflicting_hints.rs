// expect: L0110
// A second packed hint conflicts (rustc E0634).
#[repr(packed(2), packed(4))]
struct S {
    a: u8,
}

fn main() {
    let s = S { a: 1 };
    println!("{}", s.a);
}
