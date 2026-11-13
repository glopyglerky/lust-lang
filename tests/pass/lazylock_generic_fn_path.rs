// `LazyLock::new(f::<T>)`: a generic fn path as the init fn (ts-rust api/encoder/encoder.rs NODE_INDEX_TABLE_KEY).
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
static C: AtomicU64 = AtomicU64::new(0);
#[derive(Debug)]
struct Key<T> { k: u64, _t: PhantomData<T> }
unsafe impl<T> Sync for Key<T> {}
unsafe impl<T> Send for Key<T> {}
fn new_key<T>() -> Key<T> { Key { k: C.fetch_add(1, Ordering::SeqCst) + 1, _t: PhantomData } }
fn pair<A, B>() -> (Option<A>, Option<B>) { (None, None) }
static K1: LazyLock<Key<Rc<Vec<u8>>>> = LazyLock::new(new_key::<Rc<Vec<u8>>>);
static K2: LazyLock<Key<u8>> = LazyLock::new(new_key::<u8>);
static P: LazyLock<(Option<u8>, Option<String>)> = LazyLock::new(pair::<u8, String>,);
fn main() { println!("{} {} {:?}", K2.k, K1.k, *P); }
