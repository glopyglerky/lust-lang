// expect: L0110
fn inc(x: i64) -> i64 {
    x + 1
}
fn neg(x: i32) -> i32 {
    -x
}
fn main() {
    let f = if true { inc } else { neg };
    println!("{}", f(1));
}
