// expect: L0111
// rustc E0283: nothing fixes the type `Box::from` builds.
fn main() {
    let s = "x";
    let b = Box::from(s);
    println!("{}", b);
}
