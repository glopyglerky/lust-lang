struct S {
    x: i64,
}

impl S {
    fn get(&self) -> i64 {
        self.x + bonus()
    }
}

fn bonus() -> i64 {
    10
}

pub fn run() -> i64 {
    S { x: 1 }.get()
}
