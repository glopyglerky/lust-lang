// An array of non-Copy elements by value through `rev`
// (fx-array-iter).
fn main() {
    let a = [String::from("x"), String::from("y")];
    for s in a.into_iter().rev() {
        println!("{}", s);
    }
}
