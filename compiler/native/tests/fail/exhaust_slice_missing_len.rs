// expect: L0115
fn f(v: &[u8]) -> u8 {
    match v {
        [] => 0,
        [_, _, ..] => 2,
    }
}
fn main() {
    println!("{}", f(&[1]));
}
