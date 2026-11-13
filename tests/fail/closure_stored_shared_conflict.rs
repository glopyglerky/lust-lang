// expect: L0303
// rustc E0502: the stored closure reads `s` while it is mutated.
struct Holder<F> {
    f: F,
}

fn main() {
    let mut s = String::from("xy");
    let h = Holder { f: || s.len() };
    s.push('z');
    println!("{}", (h.f)());
}
