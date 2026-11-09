// capability: rejected: unions, also inside function bodies
// expect: L0602

union U {
    a: u32,
    b: f32,
}

fn main() {
    let u = U { a: 1 };
    let _ = u;
}
