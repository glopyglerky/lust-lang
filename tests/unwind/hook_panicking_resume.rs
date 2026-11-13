// A hook sees thread::panicking() true; resume_unwind at the end of main
// exits 101 without the hook.
struct D;
impl Drop for D { fn drop(&mut self) { println!("drop D {}", std::thread::panicking()); } }
fn main() {
    std::panic::set_hook(Box::new(|info| { println!("hook sees panicking={} {}", std::thread::panicking(), info.payload_as_str().unwrap_or("-")); }));
    let _d = D;
    let r = std::panic::catch_unwind(|| { let _e = D; panic!("x"); });
    println!("after {} {}", r.is_err(), std::thread::panicking());
    std::panic::resume_unwind(r.unwrap_err());
}
