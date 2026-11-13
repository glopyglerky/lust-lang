// `self` in an enum group binds the enum name itself.
mod a {
    pub enum E {
        V(i32),
        W,
    }
}
use a::E::{self, V};

fn main() {
    let e: E = V(41);
    match e {
        E::V(n) => println!("{}", n + 1),
        E::W => println!("w"),
    }
}
