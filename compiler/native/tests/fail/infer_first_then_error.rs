// expect: L0610
// rustc E0282 then E0599: an inference failure before any other error is
// still reported.
fn main() {
    let f = |x| x.len();
    let v = vec![1u8];
    v.no_such_method();
    println!("{}", f(String::new()));
}
