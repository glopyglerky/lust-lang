// Valid anonymous consts that are now type- and borrow-checked as bodies
// (Sol review 3, L2 and L3): array lengths in fields, signatures, aliases
// and impl headers, const arguments and defaults, discriminants, and
// type-position consts in a body.

mod m {
    pub const K: usize = 3;
    pub struct S;
    impl S {
        pub const Q: usize = 2;
    }
}

struct P {
    a: [u8; m::K + 1],
    b: [u8; {
        let n = 1;
        n + m::S::Q
    }],
}
fn sig(a: [u8; { let r = m::K; r * 2 }]) -> usize {
    a.len()
}
type A = [u16; { if m::K > 2 { 5 } else { 1 } }];
struct W<const N: usize = { let d = 2; d * 3 }>;
impl<const N: usize> W<N> {
    fn n(&self) -> usize {
        N
    }
}
type Wa = W<{ let x = m::K; x + 10 }>;
#[repr(u8)]
enum E {
    A = { let x = 3u8; x * 2 },
    B = 1 + 2,
}
struct Q<T>(T);
impl Q<[u8; { 2 + 2 }]> {
    fn n() -> usize {
        4
    }
}
fn f<const N: usize>() -> usize {
    N
}

fn main() {
    println!("{}", std::mem::size_of::<P>());
    println!("{}", sig([0; 6]));
    println!("{}", std::mem::size_of::<A>());
    let w: W = W;
    let wa: Wa = W;
    println!("{} {}", w.n(), wa.n());
    println!("{} {}", E::A as u8, E::B as u8);
    println!("{}", Q::<[u8; 4]>::n());
    const LOCAL: usize = 2;
    struct Inner {
        a: [u8; LOCAL * 3],
    }
    let v: [u8; { let k = LOCAL; k + 1 }] = [0; 3];
    println!("{} {}", std::mem::size_of::<Inner>(), v.len());
    println!("{}", f::<{ let a = 4; a * a }>());
}
