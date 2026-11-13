// `array::IntoIter` of non-Copy elements by value (fx-array-iter):
// every element moves out once.
fn main() {
    let v: Vec<String> = [String::from("a"), String::from("b")].into_iter().collect();
    println!("{:?}", v);
}
