// capability: rejected: #[test] functions inside impl blocks and traits (rustc rejects them too)
// expect: L0402

struct S;

impl S {
    #[test]
    fn t() {}
}

fn main() {}
