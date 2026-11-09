// `std::array::from_fn` with an element that has no drop glue (lower/iter.rs builds a plain
// aggregate, no Vec): the callback runs in index order, a 64-element array of a small enum
// (the shape of ts-rust's new_page) is built, and a panic in the callback drops only the
// closure, which owns a value with a Drop.
struct Noisy(u8);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Slot {
    Used(u8),
    Gone,
}
fn main() {
    let order = std::cell::RefCell::new(Vec::new());
    let a: [u32; 5] = std::array::from_fn(|i| {
        order.borrow_mut().push(i);
        (i * i) as u32
    });
    println!("{a:?} {:?}", order.borrow());
    let page: [Slot; 64] = std::array::from_fn(|i| if i % 7 == 0 { Slot::Used(i as u8) } else { Slot::Gone });
    println!("{} {:?} {:?}", page.iter().filter(|s| **s == Slot::Gone).count(), page[0], page[63]);
    let r = std::panic::catch_unwind(|| {
        let owned = Noisy(7);
        let b: [u16; 6] = std::array::from_fn(move |i| {
            let _keep = &owned;
            if i == 3 {
                panic!("boom at {i}");
            }
            i as u16
        });
        println!("{b:?}");
    });
    println!("caught {}", r.is_err());
}
