// expect: L0612
// rustc E0277: a type parameter is `Send` only through a bound.
fn need<X: Send>(x: X) -> X {
    x
}

fn f<T>(t: T) -> T {
    need(t)
}

fn main() {
    println!("{}", f(1u8));
}
