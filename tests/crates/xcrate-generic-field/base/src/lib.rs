pub struct S {
    pub(crate) y: i32,
    x: i32,
}
impl S {
    pub fn new() -> S {
        S { y: 1, x: 2 }
    }
    pub fn get<T>(&self, t: T) -> i32 {
        let _ = &t;
        self.x + self.y
    }
}
pub fn read_crate<T>(s: &S, t: T) -> i32 {
    let _ = &t;
    s.y
}
