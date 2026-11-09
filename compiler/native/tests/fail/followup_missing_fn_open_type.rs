// expect: L0101
// rustc E0425 only: the missing fn leaves its result's type open, and the
// field use and Debug of that open type are follow-ups, not more errors.
#![allow(unused)]
use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;
#[derive(Clone, Debug, Default)]
struct R { meta: u8, parse: Option<String> }
struct Q { roots: Option<Vec<String>> }
struct Sh { queue: Mutex<Q> }
struct Pool { shared: Sh }
impl Pool { fn start(n: u8) -> Pool { Pool { shared: Sh { queue: Mutex::new(Q { roots: None }) } } } }
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> { m.lock().unwrap() }
struct C;
impl C { fn takes(&self, a: u8, v: Vec<u8>) -> usize { v.len() + a as usize } }

fn main() {
    let (mut result, names) = match get(3) { Some((result, names)) => (Some(result), names), None => (None, None) }; if let Some(r) = &mut result { r.meta += 1; println!("{}", r.meta); } println!("{:?}", names);
}
