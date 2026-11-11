fn helper(value: i32) -> i32 {
    value + 5
}

pub fn combine(first: i32, second: i32) -> i32 {
    helper(first) * 2 + second
}

pub fn unit_argument(_: (), value: i32) -> i32 {
    value
}

pub fn unit_result() {}

pub fn bool_roundtrip(value: bool) -> bool {
    value
}

pub fn char_roundtrip(value: char) -> char {
    value
}

pub fn f32_roundtrip(value: f32) -> f32 {
    value
}

pub fn f64_roundtrip(value: f64) -> f64 {
    value
}

pub fn signed_roundtrip(value: i128) -> i128 {
    value
}

pub fn unsigned_roundtrip(value: u128) -> u128 {
    value
}
