// An unused import of a real std item or module that lust does not
// model is valid Rust with the same output, so it stays accepted (the
// ts-rust slices import several). A path std 1.99 lacks is L0506.
#![allow(unused_imports)]
use std::backtrace::Backtrace;
use std::backtrace::*;
use std::sync::mpsc::TrySendError;

fn main() {
    println!("hi");
}
