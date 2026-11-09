// expect: L0524
#[non_exhaustive()]
struct S {
    x: i64,
}

fn main() {
    let _s = S { x: 1 };
    println!("x");
}
