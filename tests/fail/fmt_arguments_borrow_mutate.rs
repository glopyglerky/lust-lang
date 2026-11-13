// expect: L0312
// An `Arguments` borrows its format arguments: no assignment to a
// borrowed argument while the value is still used.
fn main() {
    let mut x = 1;
    let a = format_args!("val {}", x);
    x = 2;
    println!("{}", a);
}
