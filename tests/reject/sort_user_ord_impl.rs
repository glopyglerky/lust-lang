// expect: L0612
// lust's sort compares by structure; a hand-written Ord impl is not
// called yet.
use std::cmp::Ordering;
#[derive(PartialEq, Eq, Debug)]
struct Q(i32);
impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        o.0.cmp(&self.0)
    }
}
fn main() {
    let mut v = vec![Q(1), Q(3), Q(2)];
    v.sort();
    println!("{:?} {:?}", v, Q(1).cmp(&Q(2)));
}
