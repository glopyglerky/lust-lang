// Comparisons on params under a PartialOrd bound (instances at scalars).
fn max2<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
fn in_order<T: PartialOrd>(a: &T, b: &T, c: &T) -> bool {
    a <= b && b <= c
}
fn main() {
    println!("{} {}", max2(3u8, 9), max2(2.5f64, -1.0));
    println!("{} {}", in_order(&1, &2, &3), in_order(&'b', &'a', &'c'));
}
