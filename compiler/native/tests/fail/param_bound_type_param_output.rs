// expect: L0307
// rustc E0515: `F: Fn(&u8) -> &u8` passed for `G: Fn(T) -> R` returns a borrow of `l`.
fn keep<T, R, G: Fn(T) -> R>(g: G, x: T) -> R {
    g(x)
}

fn bad<F: Fn(&u8) -> &u8>(f: F) -> &'static u8 {
    let l = 1u8;
    keep(f, &l)
}

fn main() {}
