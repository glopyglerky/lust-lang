// expect: L0510
mod pa {
    pub struct S {
        x: i32,
    }
    pub const K: S = S { x: 7 };
}
mod pb {
    pub const _: i32 = super::pa::K.x;
}
fn main() {
    println!("x");
}
