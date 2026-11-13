// expect: L0733
// rustc E0521: a bound written `FnMut(&i32)` makes the argument
// higher-ranked; storing it in a captured Vec escapes.
fn apply<F: FnMut(&i32)>(mut f: F) { let z = 3; f(&z); }
fn main() {
    let mut out: Vec<&i32> = Vec::new();
    apply(|x| out.push(x));
    println!("{}", out.len());
}
