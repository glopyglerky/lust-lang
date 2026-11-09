// Field access and explicit `*w` through user Deref impls: reads call
// `deref`, writes and `&mut` uses call `deref_mut` (the prints show which,
// in rustc's order), an index operand is a read, generic impls, and
// closures that capture the base for the call.
use std::ops::{Deref, DerefMut};
struct Late { file_index: usize, name: String }
struct Info { late: Late, id: u8 }
impl Deref for Info { type Target = Late; fn deref(&self) -> &Late { println!("deref"); &self.late } }
impl DerefMut for Info { fn deref_mut(&mut self) -> &mut Late { println!("deref_mut"); &mut self.late } }
struct G<T> { inner: T }
impl<T> Deref for G<T> { type Target = T; fn deref(&self) -> &T { &self.inner } }
fn idx(i: &Info) -> usize { i.file_index }
fn part1() {
    let mut info = Info { late: Late { file_index: 3, name: String::from("a.ts") }, id: 7 };
    println!("{} {} {}", info.file_index, info.name, info.id);
    info.file_index = 9;
    info.name.push_str("x");
    println!("{} {}", idx(&info), info.name.len());
    let r = &info.name;
    println!("{}", r);
    let g = G { inner: Late { file_index: 1, name: String::new() } };
    println!("{}", g.file_index);
    let v = vec![10, 20, 30];
    println!("{}", v[info.file_index - 8]);
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct P { x: i32 }
struct W { p: P }
impl Deref for W { type Target = P; fn deref(&self) -> &P { &self.p } }
impl DerefMut for W { fn deref_mut(&mut self) -> &mut P { &mut self.p } }
struct Name(String);
impl Deref for Name { type Target = str; fn deref(&self) -> &str { &self.0 } }
fn part2() {
    let mut w = W { p: P { x: 1 } };
    let a: P = *w;
    let r: &P = &*w;
    println!("{:?} {:?} {}", a, r, *w == P { x: 1 });
    (*w).x = 5;
    let m = &mut *w;
    m.x += 1;
    println!("{:?}", w.p);
    let n = Name(String::from("hey"));
    let s: &str = &*n;
    println!("{} {}", s, (*n).len());
}

struct Info2 { late: Late }
impl Deref for Info2 { type Target = Late; fn deref(&self) -> &Late { println!("deref"); &self.late } }
impl DerefMut for Info2 { fn deref_mut(&mut self) -> &mut Late { println!("deref_mut"); &mut self.late } }
fn part3() {
    let mut info = Info2 { late: Late { file_index: 3, name: String::from("a") } };
    let get = || info.file_index + 1;
    println!("{}", get());
    let mut set = || info.file_index = 7;
    set();
    let show = || (*info).name.len();
    println!("{} {}", show(), info.file_index);
    let mut v2 = vec![1, 2, 3];
    v2[info.file_index - 6] = 9;
    println!("{:?}", v2);
    let moved = move || info.name.len();
    println!("{}", moved());
}

fn main() {
    part1();
    part2();
    part3();
}
