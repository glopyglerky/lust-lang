// expect: L0510
mod pa {
    pub struct S {
        x: i32,
    }
    pub const K: S = S { x: 7 };
}
fn main() {
    let v = const { pa::K.x };
    println!("{}", v);
}
