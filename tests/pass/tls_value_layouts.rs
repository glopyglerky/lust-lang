// Key values of many layouts: a large struct and array (the init fn
// returns through an out pointer), 64-byte alignment, unit and ZST
// values, a tuple with u128, and an Option<Box<String>> through set and
// replace.
use std::cell::{Cell, RefCell};
#[repr(align(64))]
#[derive(Clone, Copy)]
struct Aligned(u8);
#[derive(Debug)]
struct Big { a: [u64; 20], s: String }
struct Zst;
thread_local! {
    static BIG: Big = Big { a: [7; 20], s: "big".to_string() };
    static BIG_CELL: RefCell<[u64; 16]> = RefCell::new([1; 16]);
    static ALIGNED: Cell<Aligned> = const { Cell::new(Aligned(5)) };
    static UNIT: () = ();
    static Z: Zst = Zst;
    static TUPLE: (u8, u128, i16) = (1, 2, 3);
    static OPTION_BOX: RefCell<Option<Box<String>>> = const { RefCell::new(None) };
}
fn main() {
    BIG.with(|b| println!("{} {} {}", b.a[0], b.a[19], b.s));
    BIG_CELL.with_borrow_mut(|a| a[3] = 9);
    println!("{:?}", BIG_CELL.take());
    ALIGNED.with(|a| println!("{}", a.get().0));
    UNIT.with(|u| println!("{:?}", u));
    Z.with(|_| println!("zst"));
    TUPLE.with(|t| println!("{} {} {}", t.0, t.1, t.2));
    OPTION_BOX.set(Some(Box::new("boxed".to_string())));
    OPTION_BOX.with_borrow(|o| println!("{:?}", o));
    println!("{:?}", OPTION_BOX.replace(None));
}
