// expect: L0101
fn f() -> i64 {
    macro_rules! get_b { () => { b } }
    let b = 2;
    get_b!()
}
fn main() { println!("{}", f()); }
