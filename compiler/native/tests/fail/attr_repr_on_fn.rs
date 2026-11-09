// expect: L0524
#[repr(C)]
fn f() {}

fn main() {
    f();
    println!("x");
}
