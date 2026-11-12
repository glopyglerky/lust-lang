// Nested records carry distinct boxes, including optional trailing commas.
// panic: assertion failed:
struct Point { horizontal: i64, vertical: i64 }
struct Rectangle { lower: Point, upper: Point }
fn nonempty(values: &[Rectangle]) -> bool {
    values.iter().all(|value| value.lower.horizontal < value.upper.horizontal && value.lower.vertical < value.upper.vertical)
}
fn main() {
    assert!(nonempty(&[Rectangle {
        lower: Point { horizontal: 100, vertical: 200 },
        upper: Point { horizontal: 100, vertical: 200 },
    }]));
}
