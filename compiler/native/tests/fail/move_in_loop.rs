// expect: L0301
fn take(v: Vec<i64>) -> usize {
    v.len()
}

fn main() {
    let v = vec![1, 2];
    for _ in 0..2 {
        take(v);
    }
}
