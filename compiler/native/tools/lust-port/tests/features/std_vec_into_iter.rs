// feature: std.vec_into_iter
// form: std::vec::IntoIter named as a type, in a signature and a field
struct Hold {
    it: std::vec::IntoIter<u32>,
}
fn make(v: Vec<u32>) -> std::vec::IntoIter<u32> {
    v.into_iter()
}
fn main() {
    let mut h = Hold { it: make(vec![3, 4]) };
    println!("{:?} {:?}", h.it.next(), h.it.next());
}
