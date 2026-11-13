// User modules named `core`, `std`, and `alloc` at the crate root. In the
// root they shadow the crates for plain paths; `::core` still names the
// crate, other modules still see the crates, and lust's own std source
// (std::time, process::exit, Option methods) is not affected.
mod core {
    pub fn f() -> u32 {
        7
    }
    pub mod time {
        pub struct Duration(pub u8);
    }
}
mod std {
    pub fn g() -> u32 {
        2
    }
}
mod alloc {
    pub fn h() -> u32 {
        3
    }
}

mod other {
    use core::cmp::max;
    use std::time::Duration;
    pub fn get() -> (u32, Duration) {
        (max(1, 2), Duration::from_millis(5))
    }
    pub fn crate_core() -> u32 {
        crate::core::f() + core::cmp::min(4, 9)
    }
}

use self::core::time::Duration as UserDuration;

fn main() {
    let d = ::std::time::Duration::from_millis(5);
    let e = ::core::time::Duration::from_secs(1);
    println!("{} {} {} {:?} {:?}", core::f(), std::g(), alloc::h(), d, e);
    let (m, d2) = other::get();
    println!("{} {} {}", m, d2.as_millis(), other::crate_core());
    println!("{}", UserDuration(4).0);
    let mut o = Some(1);
    let old = o.replace(2);
    let r: Result<u8, u8> = Ok(3);
    let t = o.take();
    println!("{:?} {:?} {:?} {}", old, o, t, r.unwrap());
    let v = vec![1, 2, 3];
    println!("{}", format!("{:?}", v));
    ::std::process::exit(0);
}
