// Constant-index pattern moves out of an array are disjoint: a later
// pattern may take another element, and a borrowed array can still be indexed.
fn main() {
    let arr = [String::from("a"), String::from("b")];
    let [a, _] = arr;
    let [_, b] = arr;
    let r = [String::from("x"), String::from("y")];
    let [ref c, _] = r;
    let d = &r[1];
    println!("{} {} {} {}", a, b, c, d);
}
