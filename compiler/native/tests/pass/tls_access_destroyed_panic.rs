// panic: cannot access a Thread Local Storage value during or after destruction: AccessError
// `with` on a key whose destructor already ran panics with Rust's message.
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
        if self.0 == 2 {
            FIRST.with(|f| println!("never {}", f.0));
        }
    }
}

thread_local! {
    static FIRST: D = D(1);
    static SECOND: D = D(2);
}

fn main() {
    SECOND.with(|s| println!("touch {}", s.0));
    FIRST.with(|f| println!("touch {}", f.0));
}
