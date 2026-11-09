// expect: L0733
fn f(mut a: &i64, b: &i64) {
    a = b;
    println!("{}", a);
}

fn main() {
    let x = 1;
    let y = 2;
    f(&x, &y);
}
