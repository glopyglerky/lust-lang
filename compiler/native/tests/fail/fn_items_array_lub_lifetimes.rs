// expect: L0110
// rustc E0308: the elements of an array literal join by rustc's LUB, which
// needs one fn's shared `'x` to match the pointer type's two lifetimes.
fn f_el(a: &u8, b: &u8) -> u8 { *a + *b }
fn f_sh<'x>(a: &'x u8, b: &'x u8) -> u8 { *a * *b }
fn main() { let v: [fn(&u8, &u8) -> u8; 2] = [f_el, f_sh]; println!("{}", v[1](&2, &3)); }
