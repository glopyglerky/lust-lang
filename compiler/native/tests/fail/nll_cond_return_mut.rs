// expect: L0303
fn head_mut(v: &mut Vec<i32>) -> Option<&mut i32> {
    if v.is_empty() { None } else { Some(&mut v[0]) }
}
fn get_or_push(v: &mut Vec<i32>) -> &mut i32 {
    if let Some(x) = head_mut(v) {
        return x;
    }
    v.push(7);
    &mut v[0]
}
fn main() {
    let mut v = Vec::new();
    println!("{}", get_or_push(&mut v));
}
