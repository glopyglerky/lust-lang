// expect: L0806
// rustc: core's matches! takes `$pattern:pat`, which cannot begin with `..=`.
fn main() {
    let a = 3;
    println!("{}", matches!(a, ..=0));
}
