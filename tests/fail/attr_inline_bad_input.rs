// expect: L0524
#[inline(sometimes)]
fn f() {}

fn main() {
    f();
    println!("x");
}
