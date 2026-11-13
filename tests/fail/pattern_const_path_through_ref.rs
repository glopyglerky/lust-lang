// expect: L0110
fn f(v: &i64) -> i32 {
    match v {
        i64::MIN => 1,
        _ => 2,
    }
}
fn main() {
    println!("{}", f(&5));
}
