// expect: L0001
// rustc: expected identifier, found `0`
// A tuple index names a field only with `: value`.
struct S(u8);
fn main() { let s = S { 0 }; let _ = s.0; }
