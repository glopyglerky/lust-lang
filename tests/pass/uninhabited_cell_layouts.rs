// Enum layouts over cells of an uninhabited type: Cell<!> is uninhabited (a variant holding
// only it is absent), RefCell/Mutex/RwLock of ! are not zero-sized so the variant stays, and
// MaybeUninit<!> / OnceLock<!> have a value (tf-build: corpus scan, h3-numbers lp7).
#![allow(dead_code)]
use std::cell::{Cell, RefCell};
use std::mem::MaybeUninit;
use std::sync::{Mutex, OnceLock, RwLock};
enum Never {}
enum E1 { A(Cell<Never>), B(u8) }
enum E2 { A(RefCell<Never>), B(u8) }
enum E3 { A(Mutex<Never>), B(u8) }
enum E4 { A(RwLock<Never>), B(u8) }
enum E5 { A(OnceLock<Never>), B(u8) }
enum E6 { A(MaybeUninit<Never>), B(u8) }
enum E7 { A(Cell<(u8, Never)>), B(u16) }
enum E8 { A(Option<Cell<Never>>), B(u8) }
fn main() {
    println!("{} {} {} {} {} {} {} {}", std::mem::size_of::<E1>(), std::mem::size_of::<E2>(), std::mem::size_of::<E3>(), std::mem::size_of::<E4>(),
        std::mem::size_of::<E5>(), std::mem::size_of::<E6>(), std::mem::size_of::<E7>(), std::mem::size_of::<E8>());
    println!("{} {} {}", std::mem::size_of::<Cell<Never>>(), std::mem::size_of::<Option<Cell<Never>>>(), std::mem::size_of::<OnceLock<Never>>());
}
