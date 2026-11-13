#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

pub fn owned() -> String { String::from("core and alloc") }
pub fn values() -> Vec<u8> { alloc::vec![10, 20, 12] }
