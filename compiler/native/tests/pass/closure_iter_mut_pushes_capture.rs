// Writing a captured borrow through an `iter_mut` item lands in the
// elements; the `&mut vs` borrow ends with the call.
fn main() {
    let x = String::from("x");
    let mut vs: Vec<Vec<&str>> = vec![vec![], vec![]];
    vs.iter_mut().for_each(|v| v.push(&x));
    println!("{:?}", vs);
}
