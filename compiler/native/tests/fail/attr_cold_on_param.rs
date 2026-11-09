// expect: L0524
fn f(#[cold] x: i64) -> i64 {
    x
}

fn main() {
    println!("{}", f(1));
}
