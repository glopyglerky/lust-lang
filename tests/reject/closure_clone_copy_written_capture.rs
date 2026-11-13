// expect: L0104
// Valid Rust: a closure that is not Copy clones a Copy capture through
// its written Clone impl (prints "tick clone 7"). Lust's clone glue does
// not call a Copy type's written Clone, so it rejects the clone.
#[derive(Copy)]
struct Tick(u8);
impl Clone for Tick {
    fn clone(&self) -> Tick {
        println!("tick clone {}", self.0);
        *self
    }
}
fn main() {
    let name = String::from("n");
    let t = Tick(7);
    let show = move |x: u8| {
        let s = t;
        format!("{name}{}{x}", s.0)
    };
    let show2 = show.clone();
    println!("{} {}", show(1), show2(2));
}
