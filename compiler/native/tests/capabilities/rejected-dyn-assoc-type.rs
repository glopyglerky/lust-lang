// capability: rejected: `dyn` for traits with associated types or std supertraits other than `Debug` and `Display`
// expect: L0702

trait T {
    type O;
    fn f(&self) -> u32;
}

fn g(x: &dyn T<O = u8>) -> u32 {
    x.f()
}

fn main() {}
