use core2::prelude::*;

macro_rules! add_one {
    ($e:expr) => { $e + 1 };
}

fn local(k: Kind) -> u32 {
    let b = matches!(k, Kind::A | Kind::C);
    add_one!(b as u32)
}

fn uses_dep(v: u32) -> u32 {
    twice!(v) + core2::flow(v)
}

pub fn run() {
    println!("{} {} {} {}", core2::is_b(Kind::B(3)), core2::speed(), core2::show(Kind::C), local(Kind::A) + uses_dep(4));
}
