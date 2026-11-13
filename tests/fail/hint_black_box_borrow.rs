// expect: L0311
// rustc E0505: the result of black_box(&x) borrows `x`, so `x` cannot move.
fn main() {
    let x = String::from("a");
    let r = std::hint::black_box(&x);
    drop(x);
    println!("{r}");
}
