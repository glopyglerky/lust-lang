fn helper(value: i32) -> i32 {
    value + 100
}

pub fn combine(first: i32, second: i32) -> i32 {
    helper(first) - second
}
