// expect: L0733
// rustc: captured variable cannot escape `FnMut` closure body. A
// mutable capture's borrow is not a source of the closure contract.
fn main() {
    let mut count = 0;
    let mut inc = || { count += 1; &count };
    println!("{}", inc());
}
