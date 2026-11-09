// expect: L0306
// rustc E0597: the closure in `h` borrows `s`, which dies first.
struct Holder<F: Fn() -> usize> {
    f: F,
}

fn main() {
    let h;
    {
        let s = String::from("x");
        h = Holder { f: || s.len() };
    }
    println!("{}", (h.f)());
}
