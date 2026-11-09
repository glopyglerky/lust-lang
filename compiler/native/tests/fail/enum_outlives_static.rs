// expect: L0306
// rustc E0597: `enum E<'a: 'static>` takes only `'static` borrows.
enum E<'a: 'static> { V(&'a u8) }
fn main() { let v = 3u8; let e = E::V(&v); if let E::V(x) = e { println!("{}", x); } }
