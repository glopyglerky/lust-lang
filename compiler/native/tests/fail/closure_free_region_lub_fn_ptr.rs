// expect: L0110
// rustc E0308: the arms coerce to the expected higher-ranked fn pointer,
// which closures with free argument regions are not.
fn main() {
    let c = |x| {
        let s: &str = x;
        s.len()
    };
    let d = |x| {
        let s: &str = x;
        s.len() + 1
    };
    let v: fn(&str) -> usize = if true { c } else { d };
    println!("{}", v("ab"));
}
