// expect: L0306
// rustc E0597: `wrap`'s `T: 'a` makes the box borrow `x`, which it outlives.
trait Show {
    fn show(&self) -> String;
}
impl<T: std::fmt::Debug> Show for T {
    fn show(&self) -> String {
        format!("{:?}", self)
    }
}
fn wrap<'a, T: Show + 'a>(t: T) -> Box<dyn Show + 'a> {
    Box::new(t)
}
fn main() {
    let b;
    {
        let x = 5u8;
        b = wrap(&x);
    }
    println!("{}", b.show());
}
