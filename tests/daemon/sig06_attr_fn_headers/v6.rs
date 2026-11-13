#![allow(clippy::all)]

/// Doubles.
#[allow(dead_code)]
#[inline]
fn twice(x: u32) -> u64 {
    x as u64 * 2
}

#[must_use]
fn third(x: u64) -> u32 {
    (x / 3) as u32
}

fn main() {
    println!("{} {}", twice(4), third(9));
}
