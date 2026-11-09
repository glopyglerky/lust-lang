// expect: L0110
// rustc: defaults for generic parameters are not allowed here.
fn f<T = i32>(x: T) -> T {
    x
}

fn main() {
    println!("{}", f(1));
}
