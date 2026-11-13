// `collect` into a user type is its `FromIterator::from_iter(self)`
// (same route as std source collections): a plain user collection
// and one whose item comes through an associated type (`A::Item`,
// the SmallVec shape).
struct Bag(Vec<u32>);
impl FromIterator<u32> for Bag {
    fn from_iter<I: IntoIterator<Item = u32>>(iterable: I) -> Bag {
        let mut v = Vec::new();
        for x in iterable {
            v.push(x * 2);
        }
        Bag(v)
    }
}

trait Arr {
    type Item;
}
impl Arr for [u16; 3] {
    type Item = u16;
}
struct Mini<A: Arr>(Vec<A::Item>);
impl<A: Arr> FromIterator<A::Item> for Mini<A> {
    fn from_iter<I: IntoIterator<Item = A::Item>>(iterable: I) -> Mini<A> {
        let mut v = Vec::new();
        for x in iterable {
            v.push(x);
        }
        Mini(v)
    }
}

fn main() {
    let b: Bag = (0..4u32).collect();
    let m: Mini<[u16; 3]> = (5..8u16).collect();
    println!("{:?} {:?}", b.0, m.0);
}
