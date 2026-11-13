// The deny-by-default lints lust checks, where rustc does not fire them:
// lifted by a level on the fn, the statement, or the item; a Copy
// ManuallyDrop takes rustc's dropping_copy_types warning; a guard behind
// a reference or in an Option is no lock lint.
#![allow(dropping_copy_types, dropping_references)]
use std::mem::ManuallyDrop;
use std::sync::{Mutex, RwLock};

// Not Copy and owns no heap memory, so a forgotten value leaks nothing.
struct Nc(u8);

#[allow(undropped_manually_drops)]
fn fn_level() {
    drop(ManuallyDrop::new(Nc(1)));
}

#[macro_export(local_inner_macros)]
macro_rules! seven { () => { 7 } }

#[allow(invalid_macro_export_arguments)]
#[macro_export(other)]
macro_rules! eight { () => { 8 } }

trait T {
    #[allow(patterns_in_fns_without_body)]
    fn f(mut x: u8);
    fn g(_: u8);
}

struct S;
impl S {
    #[deprecated]
    fn old(&self) -> u8 { 1 }
}

fn main() {
    fn_level();
    let m = ManuallyDrop::new(5);
    drop(m);
    #[allow(undropped_manually_drops)]
    drop(ManuallyDrop::new(Nc(2)));
    drop(&ManuallyDrop::new(Nc(3)));
    drop(vec![ManuallyDrop::new(6)]);
    let lock = Mutex::new(1);
    let _g = lock.lock();
    drop(_g);
    let other = Mutex::new(3);
    let _ = &other.lock();
    let _ = Some(lock.lock());
    #[allow(let_underscore_lock)]
    let _ = lock.lock();
    let rw = RwLock::new(2);
    #[allow(let_underscore)]
    let _ = rw.read();
    match lock.lock() {
        _ => {}
    }
    #[allow(deprecated)]
    let o = S.old();
    let s = "a\u{202e}b";
    println!("{} {} {} {}", seven!(), eight!(), o, s.len());
}
