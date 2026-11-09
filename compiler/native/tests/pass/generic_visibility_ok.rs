// Generic bodies check in the module of their owning def, and so
// do their instances: a generic method in a non-root module reads
// its own type's private fields, a child module reads its parent
// module's private items, and a trait default body keeps its
// trait's module when the impl lives elsewhere.
mod a {
    pub struct S {
        pub shown: i32,
        x: i32,
    }
    pub struct Tup(i32, pub i32);
    pub struct W<U> {
        pub v: U,
        sec: i32,
    }
    impl S {
        pub fn new() -> S {
            S { shown: 1, x: 2 }
        }
        fn secret(&self) -> i32 {
            9
        }
        pub fn get_g<T>(&self, t: T) -> i32 {
            let _ = &t;
            self.x + self.secret()
        }
        pub fn get_c<T>(&self, t: T) -> i32 {
            let _ = &t;
            let f = || self.x;
            f()
        }
    }
    impl<U> W<U> {
        pub fn new(v: U) -> W<U> {
            W { v, sec: 8 }
        }
        pub fn sec_g(&self) -> i32 {
            self.sec
        }
    }
    pub fn same_lit<T>(t: T) -> i32 {
        let _ = &t;
        let s = S { shown: 1, x: 3 };
        let S { x, .. } = s;
        let Tup(v, _) = Tup(4, 5);
        x + v
    }
    pub trait DefTr {
        fn dm(&self) -> i32 {
            S::new().x
        }
    }
    pub mod inner {
        pub struct Deep {
            pub(super) up: i32,
            pub(in crate::a) ina: i32,
        }
        impl Deep {
            pub fn new() -> Deep {
                Deep { up: 1, ina: 2 }
            }
        }
        pub fn child_read<T>(s: &super::S, t: T) -> i32 {
            let _ = &t;
            s.x
        }
    }
    pub fn from_a<T>(t: T) -> i32 {
        let _ = &t;
        let d = inner::Deep::new();
        d.up + d.ina
    }
}

mod b {
    pub enum E {
        V { n: i32 },
        U,
    }
    pub fn mk() -> E {
        E::V { n: 4 }
    }
}

use a::DefTr;
struct R;
impl DefTr for R {}

fn enum_pick<T>(e: &b::E, t: T) -> i32 {
    let _ = &t;
    match e {
        b::E::V { n } => *n,
        b::E::U => 0,
    }
}

fn main() {
    let s = a::S::new();
    let w = a::W::new(1);
    let e = b::mk();
    println!(
        "{} {} {} {} {} {} {}",
        s.get_g(1),
        s.get_c(1),
        w.sec_g(),
        a::same_lit(1),
        a::inner::child_read(&s, 1),
        a::from_a(1),
        R.dm() + enum_pick(&e, 1)
    );
}
