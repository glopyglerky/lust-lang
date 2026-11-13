// expect: L0306
// rustc E0597: the clone holds `&x` like the original; `x` dies first.
#[derive(Clone, Debug)]
struct S<'a> { r: &'a String }
fn main() {
    let c;
    {
        let x = String::from("x");
        let s = S { r: &x };
        c = s.clone();
    }
    println!("{:?}", c);
}
