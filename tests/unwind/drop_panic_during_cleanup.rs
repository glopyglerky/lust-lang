// A panic while a destructor runs during cleanup aborts with Rust's
// "panic in a destructor during cleanup" (exit 134).
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); if self.0 == "p" { panic!("in drop"); } } }
fn main() {
    let _a = D("a");
    let _p = D("p");
    let _b = D("b");
    let v: Vec<i32> = vec![];
    println!("start");
    let _ = v[3];
}
