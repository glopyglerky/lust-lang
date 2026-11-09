// ts-rust program.rs send_checker_barrier: `unwrap_or_else(PoisonError::into_inner)`
// gives exactly the guard, so its borrow of `token` is not `'static`.
use std::sync::{Arc, Mutex, PoisonError};
#[derive(Default)]
struct PoolToken(Mutex<Vec<Box<dyn Send>>>);
struct Emit { token: Arc<PoolToken> }
fn barrier<T: Send + 'static>(signal: impl Fn() -> T, emit: &mut Emit) -> usize {
    let token = std::mem::take(&mut emit.token);
    token
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(Box::new(signal()));
    1
}
fn main() { let mut e = Emit { token: Arc::default() }; println!("{}", barrier(|| 5u8, &mut e)); }
