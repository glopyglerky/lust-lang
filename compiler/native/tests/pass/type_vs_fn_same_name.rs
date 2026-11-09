type X = i64;
fn X() -> X {
    4
}
fn main() {
    let v: X = X();
    println!("{}", v);
}
