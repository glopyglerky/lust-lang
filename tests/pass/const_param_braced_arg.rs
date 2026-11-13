// A const parameter in braces as a const argument (`S<{ N }>`) is the
// parameter itself.
struct S<const N: usize> {
    v: [u8; N],
}

fn f<const N: usize>() -> S<{ N }> {
    S { v: [7; N] }
}

fn main() {
    let s = f::<3>();
    println!("{:?} {}", s.v, s.v.len());
}
