// expect: L0301
// rustc E0382: arr[1].1 goes through an index expression, so it may reach
// the moved arr[0].0.
fn main() {
    let arr = [(String::from("a"), 1), (String::from("b"), 2)];
    let [(a, _), _] = arr;
    println!("{} {}", a, arr[1].1);
}
