// A generic fn's opaque `impl Fn*` (a closure of its instance) inside a
// generic std type. A warm daemon build once lost the `Option::take`
// instance: the first discovery check of `main` stored its requests
// while the definer's closures were still pending (scripts/warm-check.sh).
fn wrap<T: Copy>(t: T) -> impl FnOnce() -> T {
    move || t
}

fn eat<F: FnOnce() -> R, R>(f: F) -> R {
    f()
}

fn main() {
    let mut o = Some(wrap(true));
    if let Some(f) = o.take() {
        println!("{}", f());
    }
    if let Some(f) = o.take() {
        println!("{}", f());
    }
    let w = Some(wrap(7u8));
    println!("{}", eat(w.unwrap()));
}
