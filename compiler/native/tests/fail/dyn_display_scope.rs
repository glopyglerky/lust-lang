// expect: L0306
// rustc E0597: `t` ends with the block while `d` points at it.
fn main() {
    let d: &dyn std::fmt::Display;
    {
        let t = String::from("a");
        d = &t;
    }
    println!("{}", d);
}
