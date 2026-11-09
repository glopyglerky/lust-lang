// expect: L0311
// rustc E0505: `s` moves while the stored closure borrows it.
struct Holder<F: Fn() -> usize> {
    f: F,
}

fn main() {
    let s = String::from("xy");
    let h = Holder { f: || s.len() };
    drop(s);
    println!("{}", (h.f)());
}
