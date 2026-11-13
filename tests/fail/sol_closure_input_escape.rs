// expect: L0733
// Sol review probe `closure_input_escape`; rustc: E0521 borrowed data escapes outside of closure
fn main() {
    let mut out: Vec<&i32> = Vec::new();
    let mut f = |x: &i32| out.push(x);
    let x = 1;
    f(&x);
    println!("{}", out[0]);
}
