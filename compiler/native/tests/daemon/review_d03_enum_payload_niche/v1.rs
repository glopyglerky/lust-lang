use std::mem::size_of;
#[derive(Debug, Clone, Copy)]
enum E { A(u32), B }
fn pick(i: u8) -> Option<E> { match i { 0 => None, 1 => Some(E::B), _ => Some(E::A(1)) } }
fn main() { println!("{}", size_of::<Option<E>>()); let mut i = 0; while i < 3 { println!("{:?}", pick(i)); i += 1; } }
