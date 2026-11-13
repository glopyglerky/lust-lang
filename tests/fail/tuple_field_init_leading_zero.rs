// expect: L0001
// rustc E0560: a tuple struct literal names fields by written text; `00` is no field.
struct S(u8, u8);
fn main() { let s = S { 00: 1, 1: 2 }; println!("{}", s.0); }
