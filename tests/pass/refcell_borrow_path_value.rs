// `RefCell::borrow` and `borrow_mut` by path and as fn values
// (ts-rust: `self.testing_data.as_ref().map(RefCell::borrow)`,
// `RefCell::borrow_mut(set.get_or_insert_with(Default::default))`). The
// guard borrows the cell, as the method's does.
use std::cell::{Ref, RefCell};

struct Program {
    testing: Option<RefCell<Vec<u8>>>,
}

impl Program {
    fn testing_data(&self) -> Option<Ref<'_, Vec<u8>>> {
        self.testing.as_ref().map(RefCell::borrow)
    }
}

fn main() {
    let p = Program { testing: Some(RefCell::new(vec![1, 2])) };
    if let Some(cell) = &p.testing {
        RefCell::borrow_mut(cell).push(3);
    }
    println!("{:?} {}", p.testing_data().map(|d| d.len()), RefCell::borrow(p.testing.as_ref().unwrap())[0]);
    let cells = vec![RefCell::new(1), RefCell::new(2)];
    for mut g in cells.iter().map(RefCell::borrow_mut) {
        *g *= 10;
    }
    let total: i32 = cells.iter().map(RefCell::borrow).map(|g| *g).sum();
    let none: Option<&RefCell<u8>> = None;
    println!("{} {:?}", total, none.map(RefCell::borrow).is_none());
}
