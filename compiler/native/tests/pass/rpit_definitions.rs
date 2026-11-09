// Return-position `impl Trait` definitions in plain fns and methods:
// the hidden type must meet the bounds; callers that only define and
// check are fine (using the value is a later step). Generic ones reject
// in the borrow checker for now (tests/reject/rpit_generic_definition.rs).
struct Bag {
    v: Vec<u32>,
}

impl Bag {
    fn doubled(&self) -> impl Iterator<Item = u32> {
        self.v.clone().into_iter().map(|x| x * 2)
    }
    fn total(&self) -> u32 {
        self.v.iter().sum()
    }
}

fn count_up(n: u32) -> impl Iterator<Item = u32> {
    0..n
}

fn shown() -> impl std::fmt::Display {
    5
}

fn adder() -> impl Fn(u8) -> u8 {
    |x| x + 1
}

fn main() {
    let b = Bag { v: vec![1, 2, 3, 4] };
    println!("{}", b.total());
}
