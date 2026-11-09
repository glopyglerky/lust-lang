// Associated fns, ctor variants, and fn values through type aliases of
// builtin and std types, as rustc reads `Alias::item` as `<Type>::item`:
// plain and generic aliases, aliases of aliases, default params, aliases
// in a module, `<Alias>::f`, and `Alias::<T>::f`.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

type V4 = Vec<u8>;
type VS<T> = Vec<T>;
type VP<T> = Vec<(T, u8)>;
type VV = V4;
type VD<T = u16> = Vec<T>;
type S = String;
type BX = Box<i32>;
type RC<T> = Rc<T>;
type OP = Option<u8>;
type GO<T> = Option<T>;
type RR = Result<u8, String>;
type C8 = Cell<u8>;
type RF<T> = RefCell<T>;
type AR = Arc<String>;
type MX<T> = Mutex<Vec<T>>;
type WK = std::rc::Weak<u16>;

mod m {
    pub type Bytes = Vec<u8>;
    pub type Name = String;
}
use m::Bytes;

fn main() {
    let mut v = V4::new();
    v.push(200);
    println!("{:?} {:?}", v, V4::with_capacity(3).capacity() >= 3);
    let w: Vec<i32> = VS::with_capacity(2);
    let x = VS::<u16>::from([300, 400]);
    let y = VP::from([(1i64, 2)]);
    let z = VV::from(&[9u8][..]);
    let d = <VD>::new();
    let d2: Vec<u16> = d;
    println!("{:?} {:?} {:?} {:?} {:?}", w, x, y, z, d2);
    println!("{:?} {:?}", V4::clone(&v), <V4>::new());
    let s = S::from("hi");
    let s2 = <S>::with_capacity(4);
    println!("{} {} {:?}", s, s2.len(), S::from_utf8(vec![104, 105]));
    let b = BX::new(-7);
    let r = RC::new(String::from("rc"));
    let r2 = RC::clone(&r);
    println!("{} {} {} {}", b, r2, RC::strong_count(&r), RC::ptr_eq(&r, &r2));
    let wk = RC::downgrade(&r);
    println!("{:?} {}", wk.upgrade(), RC::weak_count(&r));
    let o = OP::Some(3);
    let n = OP::None;
    let g = GO::<i8>::Some(-1);
    let h: Vec<Option<u32>> = vec![1u32, 2].into_iter().map(GO::Some).collect();
    println!("{:?} {:?} {:?} {:?} {:?}", o, n, g, h, <Option<u8>>::Some(4));
    let ok = RR::Ok(5);
    let er = RR::Err(String::from("bad"));
    println!("{:?} {:?} {:?}", ok, er, <RR>::Ok(6));
    let c = C8::new(7);
    c.set(8);
    let rf = RF::new(vec![1u8]);
    rf.borrow_mut().push(2);
    println!("{} {:?}", c.get(), rf.borrow());
    let a = AR::new(String::from("arc"));
    let a2 = AR::clone(&a);
    let mx = MX::new(vec![1i16]);
    mx.lock().unwrap().push(2);
    println!("{} {} {:?}", a2, AR::strong_count(&a), mx.lock().unwrap());
    let wk2 = WK::new();
    println!("{:?}", wk2.upgrade());
    let bytes = Bytes::from([1, 2]);
    let name = m::Name::from("mod");
    println!("{:?} {} {:?}", bytes, name, m::Bytes::new());
}
