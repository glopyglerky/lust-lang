// A generic const fn call and a generic struct literal in a static or
// const initializer take the declared type's arguments (W-STD-C; the
// `OnceLock::new()` and `PhantomData` static shapes).
struct W<T> {
    v: Option<T>,
    n: u32,
}

impl<T> W<T> {
    const fn new(n: u32) -> W<T> {
        W { v: None, n }
    }
}

struct Pair<A, B> {
    a: Option<A>,
    b: Option<B>,
}

static S: W<u8> = W::new(4);
const C: W<String> = W::new(5);
static P: Pair<u16, i8> = Pair { a: Some(3), b: None };

fn main() {
    println!("{} {} {}", S.n, S.v.is_none(), C.n);
    println!("{:?} {:?}", P.a, P.b);
}
