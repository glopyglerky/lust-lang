// `static X: LazyLock<T>` is `Sync` for `T: Send + Sync`, also when `T` holds a
// `dyn Tr` whose supertraits are `Send + Sync` (std: `F: Send`, not `F: Sync`).
use std::any::Any;
use std::sync::{Arc, LazyLock, Mutex};
trait Ev: Send + Sync + 'static {
    fn e(&self) -> String;
}
impl Ev for i32 {
    fn e(&self) -> String {
        format!("v{}", self)
    }
}
enum Repr {
    S(String),
    V(Box<dyn Ev>),
    A(Arc<dyn Any + Send + Sync>),
}
#[derive(Clone)]
struct GoError(Arc<Repr>);
fn new(s: &str) -> GoError {
    GoError(Arc::new(Repr::S(s.to_string())))
}
fn show(g: &GoError) -> String {
    match &*g.0 {
        Repr::S(s) => s.clone(),
        Repr::V(v) => v.e(),
        Repr::A(a) => format!("any u8 {}", a.downcast_ref::<u8>().is_some()),
    }
}
static E1: LazyLock<GoError> = LazyLock::new(|| new("boom"));
static E2: LazyLock<Vec<String>> = LazyLock::new(|| vec!["a".to_string()]);
static E3: LazyLock<GoError> = LazyLock::new(|| GoError(Arc::new(Repr::V(Box::new(5)))));
static E4: LazyLock<Mutex<Vec<GoError>>> = LazyLock::new(|| Mutex::new(Vec::new()));
fn need_sync<T: Sync>(_: &T) {}
fn main() {
    println!("{} {:?} {}", show(&E1), *E2, show(&E3));
    E4.lock().unwrap().push(GoError(Arc::new(Repr::A(Arc::new(1u8)))));
    E4.lock().unwrap().push(E1.clone());
    for g in E4.lock().unwrap().iter() {
        println!("{}", show(g));
    }
    let c = std::cell::Cell::new(3);
    // The init closure holds a `Cell`: `Send`, not `Sync`. std asks `F: Send`.
    let l = LazyLock::new(move || c.get() + 1);
    need_sync(&l);
    println!("{}", LazyLock::force(&l));
    let h = std::thread::spawn(|| show(&E1).len());
    println!("{}", h.join().unwrap());
}
