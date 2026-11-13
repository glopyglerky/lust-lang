// `PoisonError::into_inner` and `Weak::upgrade` as paths and
// as fn values (TRIAGE T5/T12).
use std::sync::{Arc, Mutex, PoisonError, Weak};

fn main() {
    let m = Mutex::new(3);
    {
        let g = m.lock().unwrap_or_else(PoisonError::into_inner);
        println!("{}", *g);
    }
    let h = m.lock().map_err(PoisonError::into_inner);
    println!("{}", h.is_ok());
    println!("{}", PoisonError::into_inner(PoisonError::new(9)));
    println!("{:?}", PoisonError::new(4).get_ref());
    let a = Arc::new(5);
    let w: Option<Weak<i32>> = Some(Arc::downgrade(&a));
    let v = w.as_ref().and_then(Weak::upgrade);
    println!("{:?}", v.map(|x| *x));
    println!("{:?}", Weak::upgrade(&Arc::downgrade(&a)).map(|x| *x));
}
