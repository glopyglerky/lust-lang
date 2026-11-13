// expect: L0306
// rustc E0597: the annotation's `'static` keeps `s` borrowed past its scope.
fn main() {
    let r: &'static str;
    {
        let s = String::from("hi");
        r = &s;
    }
    println!("{}", r);
}
