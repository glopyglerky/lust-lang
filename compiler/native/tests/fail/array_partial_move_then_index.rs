// expect: L0301
// rustc E0382: an index expression is not a constant index; it reads the
// whole array, and arr[0] was moved out by the pattern.
fn main() {
    let arr = [String::from("a"), String::from("b")];
    let [a, _] = arr;
    println!("{} {}", a, arr[1]);
}
