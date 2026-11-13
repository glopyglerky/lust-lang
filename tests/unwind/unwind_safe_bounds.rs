// UnwindSafe and RefUnwindSafe as written bounds.
use std::panic::{self, UnwindSafe, RefUnwindSafe};
fn run<F: FnOnce() -> i32 + UnwindSafe>(f: F) -> Result<i32, Box<dyn std::any::Any + Send>> { panic::catch_unwind(f) }
fn needs_ref<T: RefUnwindSafe>(_t: &T) {}
fn main() {
    panic::set_hook(Box::new(|_| {}));
    let v = vec![1, 2, 3];
    println!("{:?}", run(|| v.len() as i32).is_ok());
    println!("{:?}", run(|| { let w: Vec<i32> = Vec::new(); w[1] }).is_err());
    needs_ref(&5);
    needs_ref(&std::sync::Mutex::new(1));
}
