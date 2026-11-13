// expect: L0115
fn f(v: Option<&[u8]>) -> u8 {
    match v {
        None => 0,
        Some(&[]) => 1,
        Some([_, _, ..]) => 3,
    }
}
fn main() {
    println!("{}", f(Some(&[1])));
}
