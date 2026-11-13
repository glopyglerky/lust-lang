// Enum variant fields take the enum's visibility in Rust: a struct
// variant of a `pub` (or `pub(crate)`) enum in another module can be
// constructed and matched from outside that module.
mod inner {
    pub enum E {
        V { x: i32, y: i32 },
        T(i32),
        U,
    }
    pub(crate) enum C {
        W { z: i32 },
    }
}

use inner::E;

fn main() {
    let e = inner::E::V { x: 1, y: 2 };
    match e {
        inner::E::V { x, y } => println!("{} {}", x, y),
        _ => println!("no"),
    }
    let e2 = E::V { x: 9, y: 8 };
    if let E::V { x, .. } = e2 {
        println!("{}", x);
    }
    let t = E::T(4);
    println!("{}", matches!(t, E::T(4)));
    let c = inner::C::W { z: 5 };
    match c {
        inner::C::W { z } => println!("{}", z),
    }
    let u = E::U;
    println!("{}", matches!(u, E::U));
}
