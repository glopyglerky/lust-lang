// expect: L0507
// rustc E0659: the block glob `use a::*` brings the module `a::a`, so the
// glob path's own first segment is ambiguous with the module `a`.
mod a { pub mod a { pub fn f() {} } }
fn main() { { use a::*; } println!("ok"); }
