// capability: rejected: deriving traits other than Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, and Default (named in the message)
// expect: L0403

#[derive(serde::Serialize)]
struct S {
    a: u32,
}

fn main() {}
