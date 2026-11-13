// const and static Discriminant values run in the const evaluator.
use std::mem::{discriminant, Discriminant};
#[derive(Debug)] enum E { A, B(u8) }
#[repr(i8)] enum N { X = -4, Y }
const D: Discriminant<E> = discriminant(&E::B(3));
const DN: Discriminant<N> = discriminant(&N::Y);
const DO: Discriminant<Option<u8>> = discriminant(&Some(1));
const fn dd(e: &E) -> Discriminant<E> { discriminant(e) }
static S: Discriminant<E> = dd(&E::A);
fn main() { println!("{:?} {:?} {:?} {:?} {} {}", D, DN, DO, S, D == discriminant(&E::B(1)), S == dd(&E::B(0))); }
