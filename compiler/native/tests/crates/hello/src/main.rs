struct Counter {
    n: i64,
}

impl Counter {
    fn bump(&mut self, by: i64) -> i64 {
        self.n += by;
        self.n
    }
}

fn main() {
    let mut c = Counter { n: 40 };
    let v = c.bump(2);
    let mut s = String::from("hello");
    s.push_str(" crate");
    println!("{} {}", s, v);
}
