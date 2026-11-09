struct Acc {
    v: i64,
}

impl Acc {
    fn add(&mut self, x: i64) -> i64 {
        self.v += x;
        self.v
    }
}

fn helper(x: i64) -> u64 {
    x as u64 * 2
}

fn pick<T: Copy>(x: T) -> T {
    x
}

fn main() {
    let mut a = Acc { v: 1 };
    let h = helper(3);
    println!("{} {} {}", a.add(h), pick(4), helper(a.v));
}
