use std::mem::offset_of;

mod m {
    #[repr(C)]
    pub struct S {
        pub a: u8,
        b: u32,
    }
    pub struct T(pub i32, pub(crate) i32);
    pub struct W {
        pub a: i32,
        pub b: i32,
    }
    pub fn mk_w() -> W {
        W { a: 1, b: 2 }
    }
    impl S {
        pub(crate) fn get(&self) -> u32 {
            self.b
        }
        pub fn new() -> S {
            S { a: 1, b: 7 }
        }
    }
}

mod a {
    fn f() -> i32 {
        1
    }
    use self::f as g;
    pub fn h() -> i32 {
        g()
    }
}

mod b {
    pub fn f() -> i32 {
        2
    }
}
pub use b::f as bf;

fn main() {
    println!("{}", offset_of!(m::S, a));
    let s = m::S::new();
    println!("{}", s.get());
    let t = m::T(3, 4);
    let m::T(x, y) = t;
    println!("{} {}", x, y);
    let w = m::W { a: 5, ..m::mk_w() };
    println!("{} {}", w.a, w.b);
    println!("{} {}", a::h(), bf());
}
