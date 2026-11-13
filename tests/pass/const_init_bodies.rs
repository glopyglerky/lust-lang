// Const and static initializers that rustc borrow-checks and accepts: moves,
// borrows that end, extended temporaries, references to statics, and
// associated consts of generic impls and trait defaults.
struct S {
    x: u32,
}
#[derive(Clone, Copy)]
struct K {
    x: u32,
}
static A: u32 = 5;
static B: &u32 = &A;
const V: &Vec<u32> = &Vec::new();
const R: &S = &S { x: 3 };
const T: &S = {
    let _n = 1;
    &S { x: 4 }
};
const M: u32 = {
    let s = S { x: 1 };
    let t = s;
    let mut a = 2u32;
    a += t.x;
    let r = &a;
    let c = *r;
    a = 10;
    a + c
};
const P: (K, K) = {
    let k = K { x: 9 };
    (k, k)
};
struct W<T>(T);
impl<T> W<T> {
    const G: u32 = {
        let s = S { x: 2 };
        let a = s;
        a.x
    };
}
trait Tr {
    const D: u32 = {
        let s = S { x: 6 };
        s.x
    };
}
impl Tr for K {}
fn main() {
    let c = const {
        let s = S { x: 8 };
        let t = s;
        t.x
    };
    let arr = [0u8; {
        let s = S { x: 2 };
        let t = s;
        t.x as usize
    }];
    println!("{} {} {} {}", B, V.len(), R.x, T.x);
    println!("{} {} {} {} {} {}", M, P.0.x + P.1.x, W::<u8>::G, K::D, c, arr.len());
}
