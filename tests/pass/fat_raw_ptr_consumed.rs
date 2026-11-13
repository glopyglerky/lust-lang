// Raw pointers to `dyn Tr`, `str` and `[T]` from Rc::as_ptr / Arc::as_ptr and
// reference coercions, read at once: cast to a thin pointer, addr, addr_eq, eq.
use std::rc::Rc;
use std::sync::Arc;

trait Host {
    fn id(&self) -> u32;
}
struct H(u32);
impl Host for H {
    fn id(&self) -> u32 {
        self.0
    }
}

fn host_key(host: &Rc<dyn Host>) -> usize {
    Rc::as_ptr(host).cast::<()>().addr()
}

fn main() {
    let a: Rc<dyn Host> = Rc::new(H(1));
    let b = a.clone();
    let c: Rc<dyn Host> = Rc::new(H(1));
    println!("{} {}", host_key(&a) == host_key(&b), host_key(&a) == host_key(&c));
    println!("{} {}", std::ptr::addr_eq(Rc::as_ptr(&a), Rc::as_ptr(&b)), std::ptr::addr_eq(Rc::as_ptr(&a), Rc::as_ptr(&c)));
    let s: Arc<[u32]> = Arc::from(vec![1u32, 2]);
    let t = s.clone();
    println!("{}", Arc::as_ptr(&s) as *const () as usize == Arc::as_ptr(&t) as *const () as usize);
    let text = String::from("abc");
    println!("{} {}", std::ptr::eq::<str>(&text[..], &text[..]), std::ptr::eq::<str>(&text[..], &text[..2]));
    println!("{}", a.id() + c.id());
}
