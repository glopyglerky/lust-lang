// A user trait with Debug and Display supertraits: the dyn type
// implements both, so derives and formats over `Box<dyn AnyValue>`
// print the concrete value (W-BENCH3 bug 3).
use std::any::Any;
use std::fmt::{Debug, Display};
trait AnyValue: Any + Debug + Display + Send {
    fn as_any(&self) -> &dyn Any;
}
impl AnyValue for i32 {
    fn as_any(&self) -> &dyn Any { self }
}
#[derive(Debug)]
struct Wrap(u32, &'static str);
impl Display for Wrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "W{}-{}", self.0, self.1) }
}
impl AnyValue for Wrap {
    fn as_any(&self) -> &dyn Any { self }
}
trait Sub: AnyValue {}
impl Sub for Wrap {}
#[derive(Debug)]
struct Msg {
    v: Option<Box<dyn AnyValue>>,
    s: Box<dyn Sub>,
}
fn show<T: Debug + Display + ?Sized>(x: &T) -> String { format!("{x:?}|{x}") }
fn main() {
    let m = Msg { v: Some(Box::new(3)), s: Box::new(Wrap(9, "z")) };
    println!("{:?} {}", m, m.v.as_ref().unwrap().as_any().is::<i32>());
    println!("{:#?}", m.v);
    println!("{}", show(&*m.s));
    let vs: Vec<Box<dyn AnyValue>> = vec![Box::new(4), Box::new(Wrap(1, "a"))];
    for v in &vs {
        println!("{:?} {} {}", v, v, v.as_any().downcast_ref::<Wrap>().map_or(-1, |w| w.0 as i32));
    }
    let none: Option<Box<dyn AnyValue>> = None;
    println!("{:?}", none);
}
