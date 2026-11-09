// expect: L0524
#[non_exhaustive]
fn f() {}

fn main() {
    f();
    println!("x");
}
