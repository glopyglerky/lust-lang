// capability: rejected: negative impls
// expect: L0614

struct S;

impl !Send for S {}

fn main() {}
