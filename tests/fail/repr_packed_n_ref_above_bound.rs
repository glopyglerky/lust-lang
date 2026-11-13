// expect: L0651
// A reference to a field whose alignment the pack bound caps may be
// unaligned (rustc E0793): u64 needs 8, the bound is 4.
#[repr(C, packed(4))]
struct A {
    a: u8,
    c: u64,
}

fn main() {
    let a = A { a: 1, c: 3 };
    println!("{}", &a.c);
}
