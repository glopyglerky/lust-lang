// expect: L0524
#[deprecated(since = 1)]
fn f() {}

fn main() {
    f();
    println!("x");
}
