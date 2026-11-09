// expect: L0110
// rustc E0308: `h` ties its vec's element lifetime to its second input;
// the pointer type keeps them apart.
fn h<'x, 'y>(_b: &'y mut Vec<&'x u8>, _c: &'x u8) {}
fn main() { let f: fn(&mut Vec<&u8>, &u8) = h; let mut v = vec![]; f(&mut v, &1); }
