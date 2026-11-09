#![allow(clippy::all)]

/// Doubles.
#[allow(dead_code)]
#[inline]
fn twice(x: u32) -> u64 where u32: Copy {
    x as u64 * 2
}

#[must_use]
fn third(x: u64) -> u64 {
    x / 3
}

fn main() {
    println!("{} {}", twice(4), third(9));
}
