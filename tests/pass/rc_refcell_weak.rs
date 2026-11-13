// Rc<RefCell<T>> fixture set: owners with Weak back links, upgrade after drop, and counts.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Owner {
    name: String,
    gadgets: RefCell<Vec<Weak<Gadget>>>,
}

struct Gadget {
    id: u32,
    owner: Rc<Owner>,
}

fn main() {
    let owner = Rc::new(Owner { name: "Gizmo".to_string(), gadgets: RefCell::new(vec![]) });
    let g1 = Rc::new(Gadget { id: 1, owner: Rc::clone(&owner) });
    let g2 = Rc::new(Gadget { id: 2, owner: owner.clone() });
    owner.gadgets.borrow_mut().push(Rc::downgrade(&g1));
    owner.gadgets.borrow_mut().push(Rc::downgrade(&g2));
    for g in owner.gadgets.borrow().iter() {
        if let Some(g) = g.upgrade() {
            println!("{} owned by {}", g.id, g.owner.name);
        }
    }
    drop(g2);
    let alive: Vec<u32> = owner.gadgets.borrow().iter().filter_map(|w| w.upgrade()).map(|g| g.id).collect();
    println!("{:?} {} {}", alive, Rc::strong_count(&owner), Rc::weak_count(&g1));
    let w: Weak<u8> = Weak::new();
    println!("{}", w.upgrade().is_none());
}
