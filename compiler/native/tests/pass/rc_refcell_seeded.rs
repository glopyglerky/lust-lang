// 3000 seeded random operations on Rc<RefCell<T>> and Weak handles:
// clone, drop, downgrade, upgrade, borrow_mut, take, into_inner, and
// the counts and drops they print, as Rust's.
use std::cell::RefCell;
use std::rc::{Rc, Weak};
fn next(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}
#[derive(Default)]
struct Tracked {
    id: u32,
    v: Vec<u32>,
}
impl Drop for Tracked {
    fn drop(&mut self) {
        println!("drop {} {}", self.id, self.v.len());
    }
}
fn main() {
    let mut seed: u64 = 0x2545f4914f6cdd1d;
    let mut strong: Vec<Rc<RefCell<Tracked>>> = Vec::new();
    let mut weak: Vec<Weak<RefCell<Tracked>>> = Vec::new();
    let mut made = 0u32;
    for step in 0..3000u32 {
        let r = next(&mut seed);
        match r % 9 {
            0 => {
                made += 1;
                strong.push(Rc::new(RefCell::new(Tracked { id: made, v: Vec::new() })));
            }
            1 if !strong.is_empty() => {
                let i = (r >> 8) as usize % strong.len();
                let c = Rc::clone(&strong[i]);
                strong.push(c);
            }
            2 if !strong.is_empty() => {
                let i = (r >> 8) as usize % strong.len();
                strong.swap_remove(i);
            }
            3 if !strong.is_empty() => {
                let i = (r >> 8) as usize % strong.len();
                weak.push(Rc::downgrade(&strong[i]));
            }
            4 if !weak.is_empty() => {
                let i = (r >> 8) as usize % weak.len();
                match weak[i].upgrade() {
                    Some(s) => {
                        s.borrow_mut().v.push(step);
                        if (r >> 20) % 2 == 0 {
                            strong.push(s);
                        }
                    }
                    None => {
                        weak.swap_remove(i);
                    }
                }
            }
            5 if !strong.is_empty() => {
                let i = (r >> 8) as usize % strong.len();
                let s = &strong[i];
                print!("{}:{}/{} ", s.borrow().id, Rc::strong_count(s), Rc::weak_count(s));
            }
            6 if !strong.is_empty() => {
                let i = (r >> 8) as usize % strong.len();
                let taken = strong[i].take();
                println!("take {} {}", taken.id, taken.v.len());
                strong[i].borrow_mut().v.push(1);
            }
            7 if !strong.is_empty() => {
                let i = (r >> 8) as usize % strong.len();
                let s = strong.swap_remove(i);
                match Rc::into_inner(s) {
                    Some(cell) => {
                        let t = cell.into_inner();
                        println!("inner {} {}", t.id, t.v.len());
                    }
                    None => print!("shared "),
                }
            }
            8 if !weak.is_empty() => {
                let i = (r >> 8) as usize % weak.len();
                print!("w{}/{} ", weak[i].strong_count(), weak[i].weak_count());
            }
            _ => {}
        }
    }
    println!();
    println!("left {} {}", strong.len(), weak.len());
}
