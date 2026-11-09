// Empty use groups are valid and bind nothing.
use {};
use ::{};
use std::{};
use self::{};
mod a {
    pub fn b() -> i32 {
        40
    }
}
use a::{};

fn main() {
    println!("{}", a::b() + 2);
}
