trait Score {
    fn score(&self) -> i32;
}

struct Value(i32);

impl Score for Value {
    fn score(&self) -> i32 {
        self.0
    }
}

fn total<T: Score>(values: &[T]) -> i32 {
    let mut sum = 0;
    for value in values {
        sum += value.score();
    }
    sum
}

fn main() {
    let values = [Value(3), Value(4)];
    let answer = Some(total(&values) * 2);
    match answer {
        Some(value) => println!("{}", value),
        None => panic!("missing answer"),
    }
}
