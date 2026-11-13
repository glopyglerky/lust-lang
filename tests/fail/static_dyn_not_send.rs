// expect: L0612
// `Mutex<T>: Sync` needs `T: Send`; `Box<dyn Tr>` is not `Send`.
use std::sync::Mutex;
trait Tr {
    fn v(&self) -> u32;
}
static X: Mutex<Option<Box<dyn Tr>>> = Mutex::new(None);
fn main() {
    println!("{}", X.lock().unwrap().is_none());
}
