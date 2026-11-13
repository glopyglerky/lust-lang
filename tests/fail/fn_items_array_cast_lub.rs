// expect: L0110
// rustc E0308: a fn item after a fn pointer element joins by LUB, which
// needs the same lifetimes.
fn f_el(a: &u8, b: &u8) -> u8 { *a + *b }
fn f_sh<'x>(a: &'x u8, b: &'x u8) -> u8 { *a * *b }
fn main() { let v = [f_el as fn(&u8, &u8) -> u8, f_sh]; println!("{}", v[1](&2, &3)); }
