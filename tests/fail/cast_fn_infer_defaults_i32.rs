// expect: L0116
fn f(v: u8) -> u8 {
    v
}
fn main() {
    let g = f as fn(_) -> u8;
    println!("{}", g(3));
}
