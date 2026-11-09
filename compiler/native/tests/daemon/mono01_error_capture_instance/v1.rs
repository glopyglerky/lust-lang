fn first_by<'a, F: Fn(&&'a i32) -> bool>(v: &'a Vec<i32>, f: F) -> Option<&'a i32> {
    v.iter().find(f)
}
fn main() {
    let r = vec![3, 1, 4];
    println!("{:?}", first_by(&r, |x| **x > 3));
}
