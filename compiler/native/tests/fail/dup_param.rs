// expect: L0106
fn f(x: i64, x: i64) -> i64 {
    x
}

fn main() {
    println!("{}", f(1, 2));
}
