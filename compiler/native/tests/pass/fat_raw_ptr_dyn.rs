// Raw pointers to `dyn Tr` (tf-fatptr): Rc/Arc/Box::as_ptr, `&x as
// *const dyn Tr`, unsizing of `*const S`; read by `cast`, `addr`,
// `is_null` and `ptr::addr_eq` (data address only). Option uses the
// vtable word's niche, as rustc does.
use std::rc::Rc;
use std::sync::Arc;
trait Tr {
    fn f(&self) -> u32;
}
struct S(u32);
impl Tr for S {
    fn f(&self) -> u32 {
        self.0
    }
}
fn key(h: &Rc<dyn Tr>) -> usize {
    Rc::as_ptr(h).cast::<()>().addr()
}
fn pick(v: &[Rc<dyn Tr>], i: usize) -> Option<*const dyn Tr> {
    v.get(i).map(Rc::as_ptr)
}
fn main() {
    let a: Rc<dyn Tr> = Rc::new(S(1));
    let b = a.clone();
    let c: Rc<dyn Tr> = Rc::new(S(2));
    println!("{} {} {}", key(&a) == key(&b), key(&a) == key(&c), a.f() + c.f());
    let p = Rc::as_ptr(&a);
    let q: *const dyn Tr = Rc::as_ptr(&b);
    println!("{} {} {}", std::ptr::addr_eq(p, q), std::ptr::addr_eq(p, Rc::as_ptr(&c)), p.is_null());
    let ar: Arc<dyn Tr + Send + Sync> = Arc::new(S(4));
    let ar2 = Arc::clone(&ar);
    println!("{}", std::ptr::addr_eq(Arc::as_ptr(&ar), Arc::as_ptr(&ar2)));
    let astr: Arc<str> = Arc::from("xy");
    let rs: Rc<[u32]> = Rc::from(vec![1, 2, 3]);
    println!("{} {} {}", Arc::as_ptr(&astr).cast::<u8>() == astr.as_ptr(), Rc::as_ptr(&rs).len(), Rc::as_ptr(&rs).cast::<u32>() == rs.as_ptr());
    let bx: Box<dyn Tr> = Box::new(S(3));
    let bp = Box::as_ptr(&bx);
    let bs: Box<[u8]> = vec![1, 2].into_boxed_slice();
    println!("{} {} {}", bp.cast::<u8>().is_null(), Box::as_ptr(&bs).len(), std::ptr::addr_eq(Box::as_ptr(&bs), bs.as_ptr()));
    let s = S(5);
    let d: &dyn Tr = &s;
    let dp = d as *const dyn Tr;
    let tp: *const S = &s;
    let up: *const dyn Tr = tp;
    let up2: *const dyn Tr = &s;
    println!("{} {} {}", std::ptr::addr_eq(dp, up), std::ptr::addr_eq(up, up2), up.cast::<S>() == tp);
    println!("{}", dp.cast_mut().cast_const().cast::<()>() == dp as *const ());
    let mut ms = S(6);
    let md: *mut dyn Tr = &mut ms;
    println!("{}", md.is_null());
    let v: Vec<Rc<dyn Tr>> = vec![a.clone(), c.clone()];
    let first = pick(&v, 0);
    let none = pick(&v, 5);
    println!("{} {}", first.is_some_and(|x| std::ptr::addr_eq(x, p)), none.is_none());
    let ps: Vec<*const dyn Tr> = vec![Rc::as_ptr(&a), Rc::as_ptr(&c), Rc::as_ptr(&b)];
    println!("{}", ps.iter().filter(|x| std::ptr::addr_eq(**x, p)).count());
    println!("{} {}", std::mem::size_of::<*const dyn Tr>(), std::mem::size_of::<Option<*const dyn Tr>>());
}
