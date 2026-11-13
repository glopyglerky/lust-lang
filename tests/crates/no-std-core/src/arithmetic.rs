use core::cmp::min;

pub fn sum(values: &[u16]) -> u16 {
    let mut total = 0;
    for value in values { total += min(*value, 100); }
    total
}
