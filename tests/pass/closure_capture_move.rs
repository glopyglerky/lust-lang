// A move closure owns its captures; Copy captures are copied in.
struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let a = Noisy("a");
    let b = Noisy("b");
    let k = 7u8;
    let c = move || {
        let _ = &b;
        let _ = &a;
        k + 1
    };
    println!("k still {}", k);
    println!("call {}", c());
    println!("before drop");
    drop_it(c);
    println!("end");
}
fn drop_it<F: Fn() -> u8>(_f: F) {}
