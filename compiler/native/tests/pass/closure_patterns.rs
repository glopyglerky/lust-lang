// Closure parameters are irrefutable patterns.
fn apply_pair<F: Fn((u8, u8)) -> u8>(f: F) -> u8 {
    f((3, 4))
}
fn main() {
    let sum = |(a, b): (i64, i64)| a + b;
    println!("{}", sum((1, 2)));
    let deref = |&x: &u8| x + 1;
    let v = 5u8;
    println!("{}", deref(&v));
    println!("{}", apply_pair(|(a, b)| a * b));
    let early = |x: i32| {
        if x < 0 {
            return 0;
        }
        x * 2
    };
    println!("{} {}", early(-3), early(4));
}
