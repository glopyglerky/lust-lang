// A `&mut` borrow made inside the returning branch does not block later mutation.
fn get_or_push(v: &mut Vec<i32>) -> &mut i32 {
    if !v.is_empty() {
        return &mut v[0];
    }
    v.push(7);
    &mut v[0]
}

fn main() {
    let mut v = Vec::new();
    *get_or_push(&mut v) += 1;
    println!("{}", v[0]);
}
