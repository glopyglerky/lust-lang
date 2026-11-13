mod pa {
    pub struct S {
        x: i32,
        pub y: i32,
    }
    pub struct T(pub i32, i32);
    pub struct C {
        pub(crate) x: i32,
    }
    pub const K: S = S { x: 7, y: 8 };
    pub const KT: T = T(3, 4);
    pub const KC: C = C { x: 9 };
    pub const fn get(s: &S) -> i32 {
        s.x
    }
    pub const fn mk() -> S {
        S { x: 5, y: 6 }
    }
    pub mod inner {
        pub const W: i32 = super::K.x;
        pub const fn get(s: &super::S) -> i32 {
            s.x
        }
    }
}
mod pb {
    pub const fn wrap(s: &super::pa::S) -> i32 {
        super::pa::get(s)
    }
}
const A: i32 = pa::K.y;
const B: i32 = pa::get(&pa::K);
const M: pa::S = pa::mk();
const T0: i32 = pa::KT.0;
const CR: i32 = pa::KC.x;
const CH: i32 = pa::inner::W;
const WR: i32 = pb::wrap(&pa::K);
const IG: i32 = pa::inner::get(&pa::K);
const PY: i32 = match pa::K {
    pa::S { y, .. } => y,
};
fn main() {
    let blk = const { pa::get(&pa::K) };
    println!("{} {} {} {} {} {} {} {} {} {}", A, B, M.y, T0, CR, CH, WR, IG, PY, blk);
}
