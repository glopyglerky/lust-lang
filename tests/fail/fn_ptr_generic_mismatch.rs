// expect: L0110
// rustc E0308: the fn pointer fixes `T = u8`, so a `&str` argument is a mismatch.
fn twice(x: u8) -> i32 {
    x as i32 * 2
}
fn app<T>(f: fn(T) -> i32, x: T) -> i32 {
    f(x)
}

fn main() {
    println!("{}", app(twice, "six"));
}
