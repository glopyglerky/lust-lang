// expect: L0610
// rustc E0282: `len` does not fix the element type of the vec.
fn main() {
    let n = [1, 2].iter().map(|_| Vec::new()).count();
    println!("{n}");
}
