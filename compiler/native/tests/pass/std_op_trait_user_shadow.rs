// A user trait named like a std operator trait (`Sub`) does not take
// std's `impl core::ops::Sub for Duration`: a checked std trait path
// names the std trait, so `-` on Durations still works.
use std::time::Duration;

trait Tr {
    fn go(&self) -> i32;
}

trait Sub: Tr {
    fn sub(&self) -> i32 {
        3
    }
}

struct A;
impl Tr for A {
    fn go(&self) -> i32 {
        1
    }
}
impl Sub for A {}

fn main() {
    println!("{:?}", Duration::from_secs(3) - Duration::from_millis(1500));
    println!("{} {}", A.go(), A.sub());
}
