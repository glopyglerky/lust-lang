// ARCH 2.4 step 3 and 9.3: one FromIterator candidate binds the
// element type of a `Bag<_>` ascription. Bag is the local
// Vec-like type the A1 gate asks for; std Vec lands in wave A2.
// The std `Vec<_>` form is in the same file as the oracle shape.
struct Bag<T> {
    items: Vec<T>,
}
impl<T> FromIterator<T> for Bag<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut items = Vec::new();
        for x in iter {
            items.push(x);
        }
        Bag { items }
    }
}
fn main() {
    let bag: Bag<_> = (0..4u8).map(|x| x + 1).collect();
    let vec: Vec<_> = (0..3u16).collect();
    // The element type is fixed: an arithmetic op on an element
    // must use that type, not a fresh integer variable.
    let first = bag.items[0].wrapping_add(10);
    println!("{} {} {} {}", bag.items.len(), first, vec.len(), vec[2]);
}
