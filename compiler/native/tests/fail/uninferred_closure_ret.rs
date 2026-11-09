// expect: L0610
// rustc E0282: an unused closure's return type is never fixed.
fn main() {
    let _f = |_x: i32| None;
    println!("x");
}
