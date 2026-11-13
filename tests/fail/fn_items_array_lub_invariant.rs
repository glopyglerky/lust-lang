// expect: L0110
// rustc E0308: `h1` ties its inputs under `&mut`, so it is not a subtype
// of `h2`, the array's first element.
fn h1<'x>(_a: &'x u8, _b: &'x mut Vec<&'x u8>) {}
fn h2(_a: &u8, _b: &mut Vec<&u8>) {}
fn main() { let a = [h2, h1]; let mut v = vec![]; a[0](&1, &mut v); }
