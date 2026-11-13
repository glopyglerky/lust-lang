// expect: L0001
// rustc: 2 positional arguments in format string, but there is 1 argument.
// An implicit capture (`{a}`) is never a positional argument.
fn main() {
    let a = 1;
    println!("{a} {} {}", 1);
}
