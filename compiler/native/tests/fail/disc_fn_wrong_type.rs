// expect: L0110
const fn f() -> i32 {
    4
}
enum E {
    A = f(),
}

fn main() {
    let _e = E::A;
    println!("x");
}
