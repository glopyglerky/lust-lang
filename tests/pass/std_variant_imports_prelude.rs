// Explicitly imported Option and Result variants keep working in patterns.
use std::option::Option::{None, Some};
use std::result::Result::{Err, Ok};
fn f(o: Option<i32>) -> i32 {
    match o {
        Some(v) => v,
        None => -1,
    }
}
fn g(r: Result<i32, String>) -> i32 {
    match r {
        Ok(v) => v,
        Err(_) => 0,
    }
}
fn main() {
    println!("{} {} {} {}", f(Some(3)), f(None), g(Ok(4)), g(Err(String::new())));
}
