// expect: L0311
// rustc E0505: a fn pointer at `F: Fn(T) -> R` ties `R` to `T` like the fn item it holds.
fn keep<T, R, F: Fn(T) -> R>(f: F, x: T) -> R {
    f(x)
}

fn id(x: &u8) -> Option<&u8> {
    Some(x)
}

fn main() {
    let p: fn(&u8) -> Option<&u8> = id;
    let data = vec![1u8, 2];
    let o = keep(p, &data[0]);
    drop(data);
    println!("{:?}", o);
}
