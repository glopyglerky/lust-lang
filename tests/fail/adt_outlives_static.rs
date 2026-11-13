// expect: L0306
// rustc E0597: `struct H<'a: 'static>` takes only `'static` borrows.
struct H<'a: 'static> { r: &'a u8 }
fn main() { let v = 3u8; let h = H { r: &v }; println!("{}", h.r); }
