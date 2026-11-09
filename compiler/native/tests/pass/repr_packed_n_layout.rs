// `repr(packed(N))` layout (R21): field alignments cap at N under
// the C order or the Rust reorder, exactly as rustc. The checker_p19
// `CacheHashKey` shape (`repr(C, packed(4))` over two u64 halves)
// leads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C, packed(4))]
pub struct CacheHashKey {
    pub hi: u64,
    pub lo: u64,
}

impl CacheHashKey {
    pub fn is_zero(&self) -> bool {
        self.hi == 0 && self.lo == 0
    }
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug, PartialEq)]
struct K {
    a: u8,
    b: u64,
    c: u8,
}

#[repr(packed(2))]
#[derive(Copy, Clone, Debug)]
struct R {
    a: u8,
    b: u32,
    c: u16,
    d: u8,
}

#[repr(packed(4))]
#[derive(Copy, Clone, Debug)]
struct W {
    a: [u16; 4],
    b: u8,
    c: u64,
}

#[repr(C, packed(2))]
#[derive(Copy, Clone, Debug)]
struct Inner {
    x: u32,
    y: u8,
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
struct Outer {
    a: u8,
    i: Inner,
    z: u16,
}

// A 20-byte cache entry: the key plus a 4-byte value stays 20, not 24.
#[derive(Clone, Copy, Debug)]
struct Entry {
    key: CacheHashKey,
    val: u32,
}

fn main() {
    println!(
        "key {} {} entry {} {}",
        std::mem::size_of::<CacheHashKey>(),
        std::mem::align_of::<CacheHashKey>(),
        std::mem::size_of::<Entry>(),
        std::mem::align_of::<Entry>()
    );
    let k = CacheHashKey { hi: 0x1122334455667788, lo: 9 };
    let e = Entry { key: k, val: 7 };
    println!("{:?} {} {:?}", e.key, e.key.is_zero(), CacheHashKey::default());
    println!("ord {}", CacheHashKey { hi: 1, lo: 0 } > CacheHashKey { hi: 0, lo: 9 });

    println!(
        "K {} {} b={} c={}",
        std::mem::size_of::<K>(),
        std::mem::align_of::<K>(),
        std::mem::offset_of!(K, b),
        std::mem::offset_of!(K, c)
    );
    let mut k = K { a: 1, b: 0x0102030405060708, c: 9 };
    k.b = k.b + 1;
    let b = k.b;
    println!("{:x} {:?}", b, k);

    println!(
        "R {} {} a={} b={} c={} d={}",
        std::mem::size_of::<R>(),
        std::mem::align_of::<R>(),
        std::mem::offset_of!(R, a),
        std::mem::offset_of!(R, b),
        std::mem::offset_of!(R, c),
        std::mem::offset_of!(R, d)
    );
    let r = R { a: 1, b: 70000, c: 300, d: 2 };
    println!("{:?}", r);

    println!("W {} {} a={} b={} c={}", std::mem::size_of::<W>(), std::mem::align_of::<W>(), std::mem::offset_of!(W, a), std::mem::offset_of!(W, b), std::mem::offset_of!(W, c));
    let w = W { a: [1, 2, 3, 4], b: 5, c: 6 };
    let a = w.a;
    println!("{:?} {:?}", a, w);

    let o = Outer { a: 1, i: Inner { x: 5, y: 2 }, z: 3 };
    println!(
        "O {} {} i={} z={} {:?}",
        std::mem::size_of::<Outer>(),
        std::mem::align_of::<Outer>(),
        std::mem::offset_of!(Outer, i),
        std::mem::offset_of!(Outer, z),
        o
    );
    // References to fields at or under the pack bound are allowed.
    let ir = &o.i;
    let zr = &o.z;
    println!("{} {}", ir.y, zr);
    let kk = K { a: 0, b: 0, c: 0 };
    let ar = &kk.a;
    println!("{}", ar);

    // Values survive an array round trip at the packed stride.
    let arr = [K { a: 0xaa, b: 1, c: 0xbb }, K { a: 2, b: 3, c: 4 }];
    println!("{} {:?} {:?}", std::mem::size_of_val(&arr), arr[0], arr[1]);
}
