// expect: L0731
// Valid Rust: with only early-bound lifetimes rustc takes a lifetime
// turbofish; lust rejects every explicit lifetime argument on a fn.
fn pick<'a: 'a>(x: &'a i32) -> &'a i32 {
    x
}

fn main() {
    let v = 7;
    println!("{}", pick::<'_>(&v) + pick::<'static>(&5));
}
