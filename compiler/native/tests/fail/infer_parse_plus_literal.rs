// expect: L0610
// rustc E0284: `"5".parse().unwrap() + 1` does not fix the parse target.
fn main() {
    let n: i32 = "5".parse().unwrap() + 1;
    println!("{n}");
}
