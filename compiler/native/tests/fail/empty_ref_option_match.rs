// expect: L0115
enum Void {}
fn a(o: &Option<Void>) -> u8 {
    match o {
        None => 1,
    }
}
fn main() {
    println!("{}", a(&None));
}
