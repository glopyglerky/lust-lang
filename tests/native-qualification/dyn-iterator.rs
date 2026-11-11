fn total(values: &mut dyn Iterator<Item = i32>) -> i32 {
    let mut sum = 0;
    for value in values {
        sum += value;
    }
    sum
}

fn main() {
    let mut values = [3, 4].into_iter().map(|value| value * 2);
    println!("{}", total(&mut values));
}
