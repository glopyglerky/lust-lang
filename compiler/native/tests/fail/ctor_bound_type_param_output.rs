// expect: L0311
// rustc E0505: a tuple-struct constructor at `F: Fn(T) -> R` returns its argument inside `R`.
struct W<T>(T);

fn keep<T, R, F: Fn(T) -> R>(f: F, x: T) -> R {
    f(x)
}

fn main() {
    let data = vec![1u8, 2];
    let o = keep(W, &data[0]);
    drop(data);
    println!("{}", o.0);
}
