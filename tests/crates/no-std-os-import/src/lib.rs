#![no_std]
#![forbid(unsafe_code)]

use ::std::fs::read;

pub fn probe() {
    let _ = read("unused");
}
