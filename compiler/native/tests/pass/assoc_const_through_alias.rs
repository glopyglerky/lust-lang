// An associated const through a type alias of a user Adt (ts-rust options.rs
// `pub type ResolutionMode = ModuleKind;` then `ResolutionMode::NONE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M(i32);
impl M {
    pub const NONE: M = M(0);
    pub const ES: M = M(99);
    pub const ESM: Self = Self::ES;
}
pub trait Tr {
    const C: i32;
}
impl Tr for M {
    const C: i32 = 11;
}
pub type R = M;
mod m {
    pub enum K {
        A,
        B,
    }
    impl K {
        pub const FIRST: K = K::A;
    }
    pub type Mode = K;
}
fn main() {
    let a = R::NONE;
    match R::ESM {
        R::NONE => println!("none"),
        R::ES => println!("es {:?}", a),
        _ => println!("other"),
    }
    println!("{} {}", R::C, <R as Tr>::C);
    let k = m::Mode::FIRST;
    println!("{}", matches!(k, m::K::A) && !matches!(m::K::B, m::K::A));
}
