// Every macro path form rustc 1.99 accepts that lust also supports.
extern crate alloc;
use std::fmt::Write;

#[macro_export]
macro_rules! m {
    () => {
        42
    };
}

mod a {
    macro_rules! inner {
        () => {
            7
        };
    }
    pub(crate) use inner;

    pub fn f() -> i32 {
        super::m!() + self::inner!()
    }
}

macro_rules! via_crate {
    () => {
        $crate::m!() + $crate::a::inner!()
    };
}

fn main() {
    ::std::println!("leading std");
    std::prelude::v1::println!("prelude v1");
    std::prelude::rust_2024::print!("prelude 2024\n");
    core::assert!(true);
    ::core::assert_eq!(1, 1);
    core::prelude::v1::assert_ne!(1, 2);
    let v: Vec<i32> = alloc::vec![1, 2];
    let w: Vec<i32> = ::std::vec![3];
    println!("{v:?} {w:?} {}", alloc::format!("{}", 5));
    println!("{}", core::matches!(Some(1), Some(_)));
    println!("{}", core::stringify!(a + b));
    let mut s = String::new();
    core::write!(s, "w{}", 1).unwrap();
    std::writeln!(s).unwrap();
    print!("{s}");
    println!("{} {} {}", crate::m!(), self::m!(), a::inner!());
    println!("{} {}", crate::a::inner!(), a::f());
    println!("{}", via_crate!());
}
