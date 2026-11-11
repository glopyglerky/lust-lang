trait Values {
    fn values(&self) -> impl Iterator<Item = i32>;
}

struct Pair([i32; 2]);

impl Values for Pair {
    fn values(&self) -> impl Iterator<Item = i32> {
        self.0.into_iter().map(|value| value * 2)
    }
}

fn main() {
    let pair = Pair([3, 4]);
    let answer: i32 = pair.values().sum();
    println!("{}", answer);
}
