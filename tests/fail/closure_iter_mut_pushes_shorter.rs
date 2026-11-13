// expect: L0306
// rustc E0597: the closure writes `&x` into the elements of `vs`, which
// outlive `x`.
fn main() {
    let mut vs: Vec<Vec<&str>> = vec![vec![], vec![]];
    {
        let x = String::from("x");
        vs.iter_mut().for_each(|v| v.push(&x));
    }
    println!("{:?}", vs);
}
