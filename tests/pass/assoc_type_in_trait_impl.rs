impl Iterator for Counter {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        self.n += 1;
        if self.n <= 1 {
            Some(self.n)
        } else {
            None
        }
    }
}

struct Counter {
    n: i64,
}

fn main() {
    let mut c = Counter { n: 0 };
    println!("{}", c.next().unwrap());
}
