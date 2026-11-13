// as_ref through a fn returning a borrow of its argument.
fn first(v: &Option<Vec<i32>>) -> Option<&i32> {
    match v.as_ref() {
        Some(x) => x.first(),
        None => None,
    }
}
fn main() {
    let v = Some(vec![5, 6]);
    let f = first(&v);
    println!("{:?}", f);
    let w: Option<Vec<i32>> = None;
    println!("{:?}", first(&w));
}
