// expect: L0510
mod pa {
    pub struct S {
        x: i32,
    }
    pub const K: S = S { x: 7 };
}
mod pb {
    pub const W: i32 = super::pa::K.x;
}
const V: i32 = pb::W;
fn main() {
    println!("{}", V);
}
