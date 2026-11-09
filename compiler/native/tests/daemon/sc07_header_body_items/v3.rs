mod m {
    pub fn helper(x: u32) -> u32 {
        x + 1
    }
}

pub fn outer(a: u32) -> u64 where u32: Copy {
    use std::collections::HashMap;
    const K: u64 = 7;
    #[derive(Clone, Debug)]
    struct Pt {
        x: u64,
    }
    fn inner(p: &Pt) -> u64 {
        p.x * 2
    }
    enum E {
        A,
        B(u64),
    }
    let mut h: HashMap<u32, u64> = HashMap::new();
    h.insert(1, K);
    let p = Pt { x: a as u64 + K };
    let e = if p.x > 3 { E::B(inner(&p)) } else { E::A };
    match e {
        E::A => 0,
        E::B(v) => v + h[&1] + m::helper(1) as u64,
    }
}

pub struct S;

pub fn ident<T: Copy>(t: T) -> T {
    t
}

impl S {
    pub fn meth(&self, n: u8) -> u64 {
        fn local(k: u64) -> u64 {
            k + 3
        }
        local(n as u64)
    }
}

fn main() {
    println!("{} {} {}", outer(5), S.meth(4), ident(2u8));
}
