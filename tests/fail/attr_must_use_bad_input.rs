// expect: L0524
#[must_use = 1]
fn f() -> i32 {
    1
}

fn main() {
    println!("{}", f());
}
