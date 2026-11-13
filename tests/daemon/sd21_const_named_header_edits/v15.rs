// Header edits of fns that const and static initializers name: fn pointer
// statics and consts, a const fn that returns one, LazyLock closures.
use std::sync::LazyLock;
struct Store(Vec<u32>);
impl Store {
    fn new(n: u32) -> Store where u32: Copy { Store(vec![n]) }
    fn get(&self, i: usize) -> Option<&u32> { self.0.get(i) }
}
fn f(x: u32) -> u32 { x + 1 }
static P: fn(u32) -> u32 = f;
const Q: fn(u32) -> u32 = f;
const fn make() -> fn(u32) -> u32 { f }
static M: fn(u32) -> u32 = make();
static S: LazyLock<Store> = LazyLock::new(|| Store::new(f(2)));

type St = Store;
static SA: fn(u32) -> Store = St::new;
static G: LazyLock<u32> = LazyLock::new(|| *S.get(0).unwrap_or(&0));
const N: usize = 3;
fn arr() -> [u8; N] { [0; N] }
fn w<T: Copy>(t: T) -> T { t }
fn main() {
    let (p, q, mm, sn, sa) = (P, Q, M, SA, SA);
    println!("{}", sa(5).0[0]);
    println!("{} {} {} {} {} {}", p(1), q(2), mm(3), S.0[0], *G, arr().len() + w(1));
}
