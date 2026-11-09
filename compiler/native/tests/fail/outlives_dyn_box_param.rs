// expect: L0733
// rustc E0309: `Box<dyn Obj + 'a>` from `T` needs `T: 'a`.
trait Obj { fn v(&self) -> u8; }
impl Obj for u8 { fn v(&self) -> u8 { *self } }
fn f<'a, T: Obj>(x: T, _z: &'a ()) -> Box<dyn Obj + 'a> { Box::new(x) }
fn main() { println!("{}", f(3u8, &()).v()); }
