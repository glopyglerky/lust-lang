// AssertUnwindSafe lets a closure that mutates its captures be caught;
// the mutation before the panic stays.
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {} {}", self.0, std::thread::panicking()); } }
use std::panic::{self, AssertUnwindSafe};
fn main() {
    let mut v = vec![1, 2, 3];
    let r = panic::catch_unwind(AssertUnwindSafe(|| { v.push(4); let _g = D("g"); v[10] }));
    println!("{} {:?}", r.is_err(), v);
    let mut total = 0;
    for i in 0..4 {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { if i % 2 == 1 { panic!("odd {}", i) } total += i; i * 10 }));
        match r { Ok(x) => println!("ok {}", x), Err(e) => println!("caught {}", e.downcast_ref::<String>().unwrap()) }
    }
    println!("total {}", total);
}
