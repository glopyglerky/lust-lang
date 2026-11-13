// expect: L0733
// rustc E0309: a closure that boxes `T` as `dyn Obj + 'a` needs `T: 'a`.
trait Obj { fn v(&self) -> u8; }
impl Obj for u8 { fn v(&self) -> u8 { *self } }
fn f<'a, T: Obj>(x: T, _z: &'a ()) -> Box<dyn Obj + 'a> {
    let c = move || -> Box<dyn Obj + 'a> { Box::new(x) };
    c()
}
fn main() { println!("{}", f(4u8, &()).v()); }
