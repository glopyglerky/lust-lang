// expect: L0311
// rustc E0505: `keep(wrap, &data[0])` returns `wrap`'s result, which borrows `data`; a fn item at `F: Fn(T) -> R` ties `R` to `T`.
fn keep<T, R, F: Fn(T) -> R>(f: F, x: T) -> R {
    f(x)
}

fn wrap(x: &u8) -> Option<&u8> {
    Some(x)
}

fn main() {
    let data = vec![1u8, 2];
    let o = keep(wrap, &data[0]);
    drop(data);
    println!("{:?}", o);
}
