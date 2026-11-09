// capability: rejected: inferred types (`_`) in signatures
// expect: L0602

fn f(x: _) -> u32 {
    x
}

fn main() {}
