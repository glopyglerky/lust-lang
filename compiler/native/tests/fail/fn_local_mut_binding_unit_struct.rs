// expect: L0110
// A `mut` binding may not reuse the name of a block unit struct
// (rustc E0530); a plain one is its path pattern.
fn main() {
    struct S;
    let mut S = 5u8;
    S += 1;
    let _ = S;
}
