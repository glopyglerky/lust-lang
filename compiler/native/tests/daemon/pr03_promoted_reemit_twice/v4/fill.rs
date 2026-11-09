pub struct S {
    pub x: i64,
    pub y: i64,
}
pub fn mk() -> i64 {
    let s = S { x: 40, y: 1 };
    s.x + s.y
}
pub fn filler0() -> i64 {
    0
}
pub fn filler1() -> i64 {
    1
}
