// The std Deref and DerefMut surface: explicit `deref()`/`deref_mut()` on
// String, Vec, Box, Rc, Arc, references, Cow, and the RefCell and lock
// guards (rustc's probe order through `&`, `&&`, and `Box<&T>`
// receivers), generic `T: Deref` bodies at those types, and core's one
// generic `as_deref`/`as_deref_mut` on Option and Result.
use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

struct Name(String);

impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl DerefMut for Name {
    fn deref_mut(&mut self) -> &mut str {
        self.0.as_mut_str()
    }
}

fn show<T: Deref>(t: &T) -> &T::Target {
    t.deref()
}

fn bump<T: DerefMut<Target = u8>>(t: &mut T) {
    *t.deref_mut() += 1;
}

fn total<T: Deref<Target = [u8]>>(o: &Option<T>) -> u32 {
    o.as_deref().map_or(0, |s| s.iter().map(|&b| b as u32).sum())
}

fn main() {
    let mut s = String::from("ab");
    s.deref_mut().make_ascii_uppercase();
    let v = vec![1u8, 2];
    let x = 7u8;
    let r = &x;
    println!("{} {:?} {} {} {}", s.deref(), v.deref(), (&r).deref(), (&&v).deref().len(), Box::new(&s).deref());
    let mut b = Box::new(3u8);
    bump(&mut b);
    let rc: Rc<str> = Rc::from("rc");
    let arc = Arc::new(vec![9u8]);
    let cow: Cow<str> = Cow::Borrowed("cow");
    println!("{} {} {} {:?} {}", show(&b), show(&rc), show(&cow), show(&arc), show(&&5u8));

    let m = Mutex::new(vec![1]);
    m.lock().unwrap().deref_mut().push(2);
    let rw = RwLock::new(4u8);
    let cell = RefCell::new(5u8);
    *cell.borrow_mut().deref_mut() += 1;
    println!("{:?} {} {}", m.lock().unwrap().deref(), rw.read().unwrap().deref(), cell.borrow().deref());

    let mut o = Some(Name(String::from("nm")));
    if let Some(t) = o.as_deref_mut() {
        t.make_ascii_uppercase();
    }
    let mut ov = Some(vec![1u8, 2, 3]);
    ov.as_deref_mut().unwrap()[0] = 10;
    let og = Some(rw.read().unwrap());
    println!("{:?} {:?} {} {:?} {:?}", o.as_deref(), Some(Box::new(1)).as_deref(), total(&ov), og.as_deref(), None::<Rc<u8>>.as_deref());
    drop(og);

    let mut res: Result<String, u8> = Ok(String::from("ok"));
    res.as_deref_mut().unwrap().make_ascii_uppercase();
    let err: Result<Vec<u8>, String> = Err(String::from("e"));
    println!("{:?} {:?} {:?}", res.as_deref(), err.as_deref(), Ok::<Cow<[u8]>, ()>(Cow::Owned(vec![3])).as_deref());
}
