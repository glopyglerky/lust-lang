// A written `Debug` impl with a projection bound (`A::Item: Debug`,
// the SmallVec shape) is the one `{:?}` calls: the fmt instance is
// made only when the impl's bounds hold at their normal forms, and
// the projection bound used to fail that proof, so the structural
// (field-wise) answer printed instead.
trait Arr {
    type Item;
}
impl Arr for [u32; 4] {
    type Item = u32;
}
struct Mini<A: Arr> {
    items: Vec<A::Item>,
    cap: usize,
}
impl<A: Arr> core::fmt::Debug for Mini<A>
where
    A::Item: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        let mut i = 0;
        while i < self.items.len() {
            list.entry(&self.items[i]);
            i += 1;
        }
        list.finish()
    }
}

fn main() {
    let m: Mini<[u32; 4]> = Mini { items: vec![1, 2], cap: 9 };
    println!("{:?} {}", m, m.cap);
}
