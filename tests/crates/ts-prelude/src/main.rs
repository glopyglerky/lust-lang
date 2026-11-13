mod ast;
mod checker;
mod core;
mod prelude;
mod printer;
mod user;

use crate::prelude::*;

fn main() {
    let f = NodeFactory::make();
    println!("prelude {} {} {}", f.id(), tag(Kind::Fast), user::run());
}

fn tag(k: Kind) -> i64 {
    match k {
        Kind::Fast => 5,
        Kind::Slow => 6,
    }
}
