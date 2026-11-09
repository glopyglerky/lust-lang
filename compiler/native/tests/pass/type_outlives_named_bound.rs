// `T: 'a` bounds on fns and inherent methods (tsrust T13 gopanic.rs
// `GoThread::start`, `spawn_scoped`): calls prove them, and bodies may
// unsize `T` into `dyn Tr + 'a`.
trait Show {
    fn show(&self) -> String;
}
impl<T: std::fmt::Debug> Show for T {
    fn show(&self) -> String {
        format!("{:?}", self)
    }
}
fn keep<'a, T: 'a>(r: &'a T) -> &'a T {
    r
}
fn wrap<'a, T: Show + 'a>(t: T) -> Box<dyn Show + 'a> {
    Box::new(t)
}
fn start<'a, F, T>(f: F) -> Box<dyn FnOnce() -> T + 'a>
where
    F: FnOnce() -> T + 'a,
    T: 'a,
{
    Box::new(f)
}
struct G;
impl G {
    fn scoped<'s, F, T>(&self, f: F) -> Box<dyn FnOnce() -> T + 's>
    where
        F: FnOnce() -> T + 's,
        T: 's,
    {
        start(f)
    }
}
fn main() {
    let x = 5;
    println!("{}", keep(&x));
    println!("{}", wrap(&x).show());
    let b = start(|| x * 2);
    println!("{}", b());
    println!("{}", G.scoped(|| x * 3)());
}
