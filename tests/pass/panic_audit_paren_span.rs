// An operator whose left side is parenthesized panics at the `(`.
// panic: attempt to divide by zero
fn main() {
    let n = std::env::args().count() as i32;
    let d = n - 1;
    println!("{}", (n + 4) / n);
    println!("{}", (n + 4).signum() / d);
}
