// A main that returns Err: the error prints, then the thread-local
// destructors run (after main's locals), and the exit status is 1.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) { println!("drop {}", self.0); }
}
thread_local! { static K: D = D("key"); }
fn main() -> Result<(), String> {
    let _l = D("local");
    K.with(|k| println!("touch {}", k.0));
    Err("bad".to_string())
}
