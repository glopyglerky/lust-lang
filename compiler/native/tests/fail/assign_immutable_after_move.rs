// expect: L0302
// rustc E0384: a move does not undo the first assignment of an immutable local.
fn main() {
    let a = String::from("a");
    drop(a);
    a = String::from("b");
    println!("{}", a);
}
