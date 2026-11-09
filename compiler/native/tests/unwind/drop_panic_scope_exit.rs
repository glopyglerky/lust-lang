// A drop that panics during a normal scope exit unwinds the rest:
// the other locals still drop, main exits 101, stdout is flushed.
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); if self.0 == "p" { panic!("in drop"); } } }
fn main() {
    let _a = D("a");
    let _p = D("p");
    let _b = D("b");
    print!("partial");
}
