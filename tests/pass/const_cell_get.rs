// Cell::get and Cell::into_inner are const fns and run in const items.
use std::cell::Cell;
const fn a() -> u8 { let c = Cell::new(5u8); c.get() + c.into_inner() }
const fn b(c: &Cell<u32>) -> u32 { c.get() * 2 }
const A: u8 = a();
const C: Cell<u32> = Cell::new(21);
const B: u32 = b(&C);
const D: (i8, bool) = { let c = Cell::new((3i8, true)); c.get() };
fn main() { println!("{A} {B} {D:?} {}", b(&Cell::new(4))); }
