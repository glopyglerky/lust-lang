// expect: L0733
// rustc E0309: `T: 'a` holds, but the closure boxes `T` as `dyn Obj + 'b`.
trait Obj { fn v(&self) -> u8; }
impl Obj for u8 { fn v(&self) -> u8 { *self } }
fn f<'a, 'b, T: Obj>(x: T, _y: &'a T, _z: &'b ()) -> Box<dyn Obj + 'b> {
    let c = move || -> Box<dyn Obj + 'b> { Box::new(x) };
    c()
}
fn main() { println!("{}", f(4u8, &4, &()).v()); }
