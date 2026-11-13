// `NonZero*::new` and `NonZero::get` are const fns in std: callable in a
// const fn and run by the const evaluator. ts-rust's generated
// ast_generated.rs has this `NodeList` shape.
use std::num::{NonZero, NonZeroU8, NonZeroU32};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeList(std::num::NonZeroU32);

impl NodeList {
    pub const fn new(index: u32) -> Self {
        match std::num::NonZeroU32::new(index.wrapping_add(1)) {
            Some(n) => Self(n),
            None => panic!("list table index out of range"),
        }
    }

    pub const fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

const L: NodeList = NodeList::new(5);
const I: usize = L.index();
const NONE: Option<NonZeroU8> = NonZeroU8::new(0);
const SOME: Option<NonZeroU8> = NonZeroU8::new(200);
const INFERRED: Option<NonZero<u64>> = NonZero::new(1 << 40);
const fn get_or(n: Option<NonZeroU32>, d: u32) -> u32 {
    match n {
        Some(n) => n.get(),
        None => d,
    }
}
const G: u32 = get_or(NonZeroU32::new(9), 1) + get_or(NonZeroU32::new(0), 100);
static S: NodeList = NodeList::new(41);

fn main() {
    println!("{I} {L:?} {NONE:?} {SOME:?} {INFERRED:?} {G}");
    println!("{} {}", S.index(), NodeList::new(9).index());
    let n = NodeList::new(u32::MAX - 1);
    println!("{}", n.index());
}
