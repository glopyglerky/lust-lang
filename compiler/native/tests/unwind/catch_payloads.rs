// catch_unwind returns Err with the payload; &str for a literal message,
// String for a formatted one; downcasts work; thread::panicking() is
// true in drops during the unwind and false after the catch.
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {} {}", self.0, std::thread::panicking()); } }
use std::panic;
fn boom(n: i32) -> i32 { let _d = D("boom"); if n > 0 { panic!("boom {}", n) } n }
fn main() {
    let r = panic::catch_unwind(|| boom(0));
    println!("{:?}", r.is_ok());
    let r = panic::catch_unwind(|| boom(3));
    match r { Ok(v) => println!("ok {}", v), Err(e) => println!("err {:?} {:?}", e.downcast_ref::<&str>(), e.downcast_ref::<String>()) }
    println!("panicking after {}", std::thread::panicking());
    let r = panic::catch_unwind(|| { let _x = D("x"); panic!("lit") });
    let e = r.unwrap_err();
    println!("{:?} {:?}", e.downcast_ref::<&str>(), e.downcast_ref::<String>());
}
