// A by-value use of one tuple field captures only that field.
struct Noisy(u8);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn consume(n: Noisy) -> u8 {
    n.0
}
fn main() {
    let t = (Noisy(1), Noisy(2), 3u8);
    let c = move || consume(t.0);
    println!("t.1 still here: {}", t.1 .0);
    println!("t.2 still here: {}", t.2);
    println!("call {}", c());
    println!("end of main");
}
