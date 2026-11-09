// Rust 2021 captures disjoint fields: the closure borrows `p.x`
// mutably while `p.y` stays readable outside.
struct Point {
    x: i32,
    y: i32,
    tag: String,
}
fn main() {
    let mut p = Point { x: 1, y: 2, tag: String::from("pt") };
    let mut shift = |d: i32| p.x += d;
    println!("y={} tag={}", p.y, p.tag);
    shift(10);
    shift(5);
    println!("x={} y={}", p.x, p.y);
    let t = (String::from("left"), String::from("right"));
    let take_left = move || t.0;
    println!("{}", take_left());
    let pair = (String::from("one"), 2u8);
    let first_len = || pair.0.len();
    println!("{} {}", first_len(), pair.1);
}
