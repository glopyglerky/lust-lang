// `std::any::type_name::<T>()` with rustc's exact strings: full
// defining paths, erased lifetimes (a lifetime argument shows as
// `'_`), and trailing default arguments trimmed.
use std::any::type_name;
use std::collections::{BTreeMap, HashMap};
mod inner {
    pub struct Deep<T>(pub T);
}
mod m0 {
    pub struct Stats {
        pub n: i64,
    }
}
struct S {
    #[allow(dead_code)]
    x: u8,
}
struct G<T, const N: usize>([T; N]);
struct D<T = u8>(T);
trait Tr {}
impl Tr for u8 {}

fn p<T: ?Sized>() {
    println!("{}", std::any::type_name::<T>());
}
fn short<T: ?Sized>() -> String {
    std::any::type_name::<T>().rsplit("::").next().unwrap_or("").to_string()
}

fn main() {
    p::<u8>();
    p::<i128>();
    p::<f64>();
    p::<bool>();
    p::<char>();
    p::<()>();
    p::<str>();
    p::<&str>();
    p::<&mut u8>();
    p::<&&u8>();
    p::<*const u8>();
    p::<[u8]>();
    p::<[u8; 3]>();
    p::<(u8, u16)>();
    p::<(u8,)>();
    p::<String>();
    p::<Vec<Vec<String>>>();
    p::<Option<u8>>();
    p::<Result<u8, String>>();
    p::<Box<[u8]>>();
    p::<std::rc::Rc<u8>>();
    p::<std::sync::Arc<u8>>();
    p::<std::cell::RefCell<String>>();
    p::<std::cell::Cell<u8>>();
    p::<HashMap<String, u8>>();
    p::<BTreeMap<String, Vec<u8>>>();
    p::<S>();
    p::<inner::Deep<u8>>();
    p::<G<u8, 4>>();
    p::<D>();
    p::<D<u16>>();
    p::<fn(u8) -> u16>();
    p::<fn(&u8) -> &u8>();
    p::<dyn Tr>();
    p::<Box<dyn Tr + Send>>();
    p::<std::ops::Range<usize>>();
    p::<std::cmp::Ordering>();
    p::<std::time::Duration>();
    p::<std::any::TypeId>();
    p::<std::marker::PhantomData<u8>>();
    p::<std::borrow::Cow<str>>();
    p::<m0::Stats>();
    p::<Vec<m0::Stats>>();
    p::<std::cell::OnceCell<u8>>();
    p::<std::path::PathBuf>();
    p::<std::sync::atomic::AtomicUsize>();
    p::<core::option::Option<u8>>();
    println!("{}", short::<String>());
    println!("{}", std::any::type_name::<Vec<i32>>());
    println!("{}", core::any::type_name::<i32>());
    println!("{}", type_name::<Option<String>>());
}
