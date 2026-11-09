pub const BASE: i64 = 40;

pub enum Kind {
    A,
    B(i64),
}

pub struct Thing {
    pub v: i64,
    w: i64,
}

pub fn make(v: i64) -> Thing {
    Thing { v, w: v + 1 }
}

pub fn get(t: &Thing) -> i64 {
    t.v + t.w
}

fn secret() -> i64 {
    9
}

pub fn call_secret() -> i64 {
    secret()
}
