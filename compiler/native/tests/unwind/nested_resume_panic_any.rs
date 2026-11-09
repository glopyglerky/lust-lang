// Nested catches, resume_unwind of a caught payload (no hook), and
// panic_any payloads of other types.
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {} {}", self.0, std::thread::panicking()); } }
use std::panic;
fn main() {
    let r = panic::catch_unwind(|| {
        let _a = D("a");
        let inner = panic::catch_unwind(|| { let _b = D("b"); panic!("inner") });
        println!("inner err {}", inner.is_err());
        let payload = inner.unwrap_err();
        panic::resume_unwind(payload);
    });
    let e = r.unwrap_err();
    println!("outer {:?}", e.downcast_ref::<&str>());
    let r2 = panic::catch_unwind(|| { panic::panic_any(42i32); });
    let e2 = r2.unwrap_err();
    println!("{:?} {:?}", e2.downcast_ref::<i32>(), e2.downcast_ref::<&str>());
    let r3 = panic::catch_unwind(|| { panic::panic_any(String::from("owned any")); });
    println!("{:?}", r3.unwrap_err().downcast_ref::<String>());
}
