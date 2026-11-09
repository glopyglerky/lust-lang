// A fn item that returns a `'static` borrow (and takes none) coerces to
// a fn pointer with an elided return lifetime.
fn take(f: fn(&u8) -> &str) -> usize {
    let x = 5u8;
    f(&x).len()
}

fn name(_x: &u8) -> &'static str {
    "abc"
}

fn main() {
    println!("{}", take(name));
}
