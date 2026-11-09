// capability: rejected: refutable patterns here (a const pattern in a closure parameter; rustc rejects it too, E0005)
// expect: L9001

const ONE: u32 = 1;

fn main() {
    let f = |ONE: u32| {};
    f(1);
}
