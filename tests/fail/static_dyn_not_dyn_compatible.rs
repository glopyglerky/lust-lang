// expect: L0701
// A static's type is checked for dyn compatibility like a signature.
use std::sync::Mutex;
trait NotSafe {
    fn g<T>(&self, t: T);
}
static X: Mutex<Option<Box<dyn NotSafe + Send>>> = Mutex::new(None);
fn main() {
    println!("{}", X.lock().unwrap().is_none());
}
