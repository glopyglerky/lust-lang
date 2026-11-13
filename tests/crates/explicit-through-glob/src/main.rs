mod checker;
mod prelude;

use crate::prelude::Kind;

fn tag(k: Kind) -> i64 {
    match k {
        Kind::A => 1,
        Kind::B => 2,
    }
}

fn main() {
    println!("kind {}", tag(Kind::B));
}
