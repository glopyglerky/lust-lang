// expect: L0306
// rustc E0597: the closure stores `&x` into a captured outer Vec; `x` dies first.
fn main() {
    let mut out: Vec<&i32> = Vec::new();
    {
        let x = 5;
        let mut f = |r: &i32| {
            let _ = r;
            out.push(&x);
        };
        f(&1);
    }
    println!("{:?}", out);
}
