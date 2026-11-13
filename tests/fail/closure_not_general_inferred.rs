// expect: L0735
// rustc: "implementation of `Fn` is not general enough". A `let` closure
// with no annotation has a fixed argument lifetime; `apply` needs one
// that takes any.
fn apply<F: Fn(&String) -> usize>(f: F) -> usize {
    let s = String::from("ab");
    f(&s)
}
fn main() {
    let len = |x| {
        let s: &String = x;
        s.len()
    };
    println!("{}", apply(len));
}
