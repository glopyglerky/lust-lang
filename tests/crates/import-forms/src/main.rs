mod a;
mod b;

use a::{one, two as pair};
use b::three as _;

mod inner {
    use super::*;

    pub fn run() -> i64 {
        base() + a::one()
    }
}

fn base() -> i64 {
    100
}

fn main() {
    println!("forms {}", one() + pair() + inner::run());
}
