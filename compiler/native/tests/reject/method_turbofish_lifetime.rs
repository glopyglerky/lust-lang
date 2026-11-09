// expect: L0731
// Valid Rust: rustc ignores a late-bound lifetime in a method turbofish
// (a future-compat warning); lust rejects every explicit lifetime arg.
struct S(i32);

impl S {
    fn get<'a>(&'a self) -> &'a i32 {
        &self.0
    }
}

fn main() {
    let s = S(1);
    println!("{}", s.get::<'static>());
}
