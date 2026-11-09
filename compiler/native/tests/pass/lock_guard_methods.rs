use std::sync::{Mutex, MutexGuard};
struct S { m: Mutex<Vec<i32>> }
impl S {
    fn get(&self) -> MutexGuard<'_, Vec<i32>> { self.m.lock().unwrap() }
    fn get2<'a>(&'a self) -> MutexGuard<'a, Vec<i32>> { self.m.lock().unwrap() }
}
fn main() { let s = S { m: Mutex::new(vec![]) }; s.get().push(1); s.get2().push(2); println!("{:?}", s.get()); }
