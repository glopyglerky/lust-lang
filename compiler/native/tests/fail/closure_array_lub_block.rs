// expect: L0110
// rustc E0308: an array element after the first that is a block coerces to
// the first element's closure type; it does not join as a fn pointer.
fn main() {
    let arr = [|| 1, { || 2 }];
    println!("{}", arr[1]());
}
