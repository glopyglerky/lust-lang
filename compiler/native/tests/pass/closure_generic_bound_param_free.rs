// `F: FnMut(T)` with `T = &i32`: the closure's argument is the
// caller's type (free), so it can be stored in a captured Vec.
fn apply<T, F: FnMut(T)>(t: T, mut f: F) { f(t) }
fn main() {
    let x = 5;
    let mut out: Vec<&i32> = Vec::new();
    apply(&x, |r| out.push(r));
    println!("{}", out.len());
}
