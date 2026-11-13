// Items named `h`, `C`, `S`, `P` nested in many scopes at once: free
// and generic fns, inherent and generic impl methods, trait default
// methods (plain and generic traits), closures, const initializers,
// and module fns. Each one is its own item (def path from its parent).
const C: i64 = 1;
fn h() -> i64 { 2 }

fn free() -> i64 {
    const C: i64 = 10;
    static S: i64 = 11;
    struct P(i64);
    fn h() -> i64 { C + 1 }
    h() + S + P(C).0
}

fn generic<T: Copy>(_t: T) -> i64 {
    static S: i64 = 20;
    fn h() -> i64 { C + 21 }
    h() + S
}

struct H;
impl H {
    fn m(&self) -> i64 {
        const C: i64 = 30;
        fn h() -> i64 { C }
        macro_rules! mk { () => { 31 } }
        h() + mk!()
    }
}

struct W<X>(X);
impl<X> W<X> {
    fn m(&self) -> i64 {
        static S: i64 = 40;
        fn h() -> i64 { 41 }
        h() + S
    }
}

trait T1 {
    fn d(&self) -> i64 {
        const C: i64 = 50;
        static S: i64 = 51;
        struct P(i64);
        fn h() -> i64 { C + 2 }
        h() + S + P(3).0
    }
}
trait T2<X> {
    fn d(&self, _x: X) -> i64 {
        static S: i64 = 60;
        fn h() -> i64 { C + 61 }
        h() + S
    }
}
impl T1 for u8 {}
impl T2<u8> for u16 {}
impl T2<i32> for u16 {}

const K: i64 = {
    const fn h() -> i64 { 70 }
    const C: i64 = 71;
    h() + C
};

mod m {
    pub fn f() -> i64 {
        const C: i64 = 80;
        fn h() -> i64 { C + 1 }
        h()
    }
}

fn main() {
    let c = || {
        static S: i64 = 90;
        fn h() -> i64 { 91 }
        h() + S + C
    };
    println!("{} {} {}", free(), generic(1u8) + generic(2i32), H.m());
    println!("{} {}", W(1u8).m() + W(true).m(), T1::d(&0u8));
    println!("{} {}", T2::<u8>::d(&0u16, 1u8) + T2::<i32>::d(&0u16, 2), K);
    println!("{} {} {}", m::f(), c(), h() + C);
}
