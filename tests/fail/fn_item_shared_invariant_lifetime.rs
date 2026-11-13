// expect: L0110
// rustc E0308: `'x` sits under `&mut`, so `h` cannot take two unrelated
// lifetimes there (one type is more general than the other).
fn h<'x>(_a: &'x u8, _b: &'x mut Vec<&'x u8>) {}
fn main() { let f: fn(&u8, &mut Vec<&u8>) = h; let a = 1; let mut v = vec![]; f(&a, &mut v); }
