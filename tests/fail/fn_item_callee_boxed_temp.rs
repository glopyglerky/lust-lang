// expect: L0306
// rustc E0716: the `&c` temporary dies at the end of the `let`; the
// call autoderefs `w` through it.
fn seven() -> i32 { 7 }
fn main() {
    let c = seven;
    let w = Box::new(&&c);
    println!("{}", w());
}
