// The Cell and RefCell key methods: set, get, take, replace, with_borrow,
// and with_borrow_mut. `set` on a lazy key not made yet stores the value
// without running the initializer; on a const key it replaces (and drops)
// the const value.
use std::cell::{Cell, RefCell};

#[derive(Debug, Default)]
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop D({})", self.0);
    }
}

thread_local! {
    static COUNT: Cell<u32> = const { Cell::new(0) };
    static LAZY_CELL: Cell<i64> = {
        println!("init lazy cell");
        Cell::new(-1)
    };
    static NAMES: RefCell<Vec<String>> = {
        println!("init names");
        RefCell::new(vec!["first".to_string()])
    };
    static CONST_D: RefCell<D> = const { RefCell::new(D(1)) };
    static LAZY_D: RefCell<D> = {
        println!("init lazy D");
        RefCell::new(D(10))
    };
}

fn main() {
    COUNT.set(COUNT.get() + 1);
    COUNT.set(COUNT.get() + 1);
    println!("count {}", COUNT.get());
    println!("replace {}", COUNT.replace(40));
    println!("take {}", COUNT.take());
    println!("after take {}", COUNT.get());

    LAZY_CELL.set(7);
    println!("lazy cell {}", LAZY_CELL.get());

    NAMES.set(vec!["set".to_string()]);
    NAMES.with_borrow_mut(|v| v.push("pushed".to_string()));
    NAMES.with_borrow(|v| println!("names {:?}", v));
    println!("take {:?}", NAMES.take());
    println!("replace {:?}", NAMES.replace(vec!["z".to_string()]));
    NAMES.with(|n| println!("with {:?}", n.borrow()));

    CONST_D.set(D(2));
    LAZY_D.set(D(11));
    LAZY_D.set(D(12));
    let old = LAZY_D.replace(D(13));
    println!("old {:?}", old);
    println!("taken {:?}", CONST_D.take());
    println!("end");
}
