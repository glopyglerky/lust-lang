// capability: rejected: unions, also inside function bodies
// expect: L0602

fn main() {
    union U {
        a: u32,
        b: f32,
    }
    let u = U { a: 1 };
    let _ = u;
}
