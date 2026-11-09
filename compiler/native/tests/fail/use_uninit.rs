// expect: L0310
fn main() {
    let x: i64;
    if std::env::args().count() > 5 {
        x = 1;
    }
    println!("{}", x);
}
