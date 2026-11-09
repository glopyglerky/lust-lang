// An adapter through a `&mut` of a slice iterator; the iterator is used after.
fn main() {
    let v = vec![1, 2, 3, 4];
    let mut it = v.iter();
    let r = &mut it;
    let two: Vec<&i32> = r.take(2).collect();
    println!("{:?} {:?}", two, it.next());
}
