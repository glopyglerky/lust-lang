// expect: L0306
// Only a `let` extends the arguments array of `format_args!`; an
// assignment drops it at the end of the statement (rustc E0716).
fn main() {
    let x = 4;
    let a;
    a = format_args!("{}", x);
    println!("{}", a);
}
