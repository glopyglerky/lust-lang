// A region-only signature edit (`&str` to `&'static str`, the same
// erased type) must recheck the caller's borrow check (W-BORROW).
fn store(s: &str) -> usize {
    s.len()
}

fn main() {
    let n;
    {
        let t = String::from("xyz");
        n = store(&t);
    }
    println!("{n}");
}
