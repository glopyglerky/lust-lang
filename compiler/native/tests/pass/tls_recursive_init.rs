// A lazy initializer that reads its own key: the inner initialization
// finishes first, then the outer value replaces it and the inner value
// drops (Rust 1.99 LazyStorage). The destructor registers once.
use std::cell::Cell;

struct D(u32);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
    static R: D = {
        let n = DEPTH.get();
        DEPTH.set(n + 1);
        println!("init {}", n);
        if n < 2 {
            R.with(|r| println!("inner sees {}", r.0));
        }
        D(n)
    };
}

fn main() {
    R.with(|r| println!("main sees {}", r.0));
    R.with(|r| println!("again {}", r.0));
    println!("end");
}
