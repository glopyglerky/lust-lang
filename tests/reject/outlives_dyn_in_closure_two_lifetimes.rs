// expect: L0733
// Valid Rust (`T: 'a` is implied by `&'a T`). In a closure body lust
// knows a type param outlives the closure's slots only when it outlives
// every lifetime of the fn; here it outlives `'a` but not `'b`.
trait Obj { fn v(&self) -> u8; }
impl Obj for u8 { fn v(&self) -> u8 { *self } }
fn f<'a, 'b, T: Obj>(x: T, _y: &'a T, _z: &'b ()) -> Box<dyn Obj + 'a> {
    let c = move || -> Box<dyn Obj + 'a> { Box::new(x) };
    c()
}
fn main() { println!("{}", f(4u8, &4, &()).v()); }
