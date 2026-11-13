pub struct S {
    x: i32,
}
impl S {
    pub fn new() -> S {
        S { x: 7 }
    }
    pub fn get<T>(&self, t: T) -> i32 {
        let _ = &t;
        self.x
    }
}
pub fn read<T>(s: &S, t: T) -> i32 {
    let _ = &t;
    s.x
}
