// Generic methods of an `impl dyn Trait` block (R25, ts-rust
// `impl dyn AnyValue { fn downcast_ref<T: 'static>(&self) }`), with and
// without a turbofish.
use std::any::Any;
use std::fmt::Debug;
pub trait AnyValue: Any + Debug + Send {
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any + Debug + Send> AnyValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl dyn AnyValue {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
    pub fn is<T: 'static>(&self) -> bool {
        self.as_any().is::<T>()
    }
    fn pair<A: Default + Debug, B: From<u8> + Debug>(&self, b: u8) -> (A, B) {
        (A::default(), B::from(b))
    }
}
#[derive(Debug)]
struct Req(u32);
#[derive(Debug)]
struct Resp(String);
fn kind(msg: Option<&dyn AnyValue>) -> &'static str {
    match msg {
        None => "nil",
        Some(m) if m.downcast_ref::<Req>().is_some() => "req",
        Some(_) => "resp",
    }
}
fn assert_params<T: 'static>(parsed: &Option<Box<dyn AnyValue>>) -> &T {
    match parsed.as_deref().and_then(|p| p.downcast_ref::<T>()) {
        Some(p) => p,
        None => panic!("not it"),
    }
}
fn main() {
    let a: Box<dyn AnyValue> = Box::new(Req(4));
    let b: Box<dyn AnyValue> = Box::new(Resp("x".into()));
    println!("{} {} {}", kind(Some(&*a)), kind(Some(b.as_ref())), kind(None));
    println!("{:?} {:?}", a.downcast_ref::<Req>(), a.downcast_ref::<Resp>());
    println!("{} {}", b.is::<Resp>(), b.is::<Req>());
    let p: (String, u32) = a.pair(9);
    println!("{:?} {:?}", p, a.pair::<u8, u16>(3));
    let o = Some(a);
    println!("{:?}", assert_params::<Req>(&o));
}
