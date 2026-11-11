fn helper(value: i32) -> i32 {
    value * 7
}

fn main() {
    println!("left={} right={} local={}", left::combine(10, 1), right::combine(10, 1), helper(3));
}
