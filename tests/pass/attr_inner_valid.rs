//! Inner attributes and doc comments in the positions Rust permits:
//! crate start, module and impl starts, a fn body, loop bodies, a
//! statement block, a match body, and empty bodies that hold only an
//! inner attribute.
#![allow(dead_code)]

/// Outer doc on an item.
fn documented() {}

mod m {
    //! Module docs.
    #![allow(dead_code)]
    pub fn f() {}
}

mod empty_mod {
    #![allow(dead_code)]
}

struct S;

impl S {
    #![allow(dead_code)]
    fn f() {}
}

struct E0;

impl E0 {
    #![allow(dead_code)]
}

trait Tr {
    #![allow(dead_code)]
    fn f();
}

fn main() {
    #![allow(dead_code)]
    //! Inner doc in a fn body.
    let _ = documented;
    let mut n: i64 = 0;
    while n < 1 {
        #![allow(dead_code)]
        n += 1;
    }
    loop {
        #![allow(dead_code)]
        break;
    }
    {
        #![allow(dead_code)]
        println!("block");
    }
    match n {
        #![allow(dead_code)]
        _ => println!("match"),
    }
    println!("ok {}", n);
}
