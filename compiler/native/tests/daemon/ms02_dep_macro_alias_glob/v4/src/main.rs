use std::fmt::Write as _;
mod m {
    pub use util::matches as is_it;
}
use m::is_it as matches;
#[derive(Clone, Copy)]
enum K {
    A,
    B(i64),
}

fn is_b(k: K) -> bool {
    let b = matches!(k, K::B(x) if x > 0);
    b
}

fn show(k: K) -> String {
    let mut s = String::new();
    write!(s, "{}", is_b(k) as i64).unwrap();
    s
}

fn main() {
    let n = util::helper();
    println!("{} {} {}", is_b(K::B(n)), is_b(K::A), show(K::B(1)));
}
