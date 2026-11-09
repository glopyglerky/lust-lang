// Renames of std imports bind the new name: types, fns, traits,
// modules, macros, variants, and crate roots, plain and renamed,
// at module level and function-local.
use std::cmp::max as mx;
use std::fmt::Display as D2;
use std::mem::drop as mydrop;
use std::mem::swap;
use std::option::Option as O2;
use std::option::Option::Some as S2;
use std::result::Result as R2;
use std::vec as vec2;
use std::vec::Vec as V2;

mod inner {
    pub use std::string::String as S3;
}
use inner::S3;
use std::mem as mm;
use std as s;

fn show<T: D2>(x: T) -> S3 {
    format!("{}", x)
}

fn main() {
    let a: O2<i32> = O2::Some(3);
    println!("{}", a.unwrap());
    let b = S2(4);
    match b {
        S2(v) => println!("{}", v),
        None => println!("none"),
    }
    let r: R2<i32, i32> = R2::Ok(5);
    match r {
        R2::Ok(v) => println!("{}", v),
        R2::Err(e) => println!("err {}", e),
    }

    let mut v: V2<i32> = V2::new();
    v.push(9);
    let w = vec2![1, 2];
    println!("{} {}", v[0] + w.len() as i32, show(7));

    let mut x = 1;
    let mut y = 2;
    swap(&mut x, &mut y);
    let t = S3::from("z");
    mydrop(t);
    mm::drop(String::from("q"));
    println!("{} {} {} {}", x, y, mx(x, y), s::mem::size_of::<u8>());

    use std::option::Option as Local;
    let l: Local<i32> = Local::Some(11);
    println!("{}", l.unwrap());
}
