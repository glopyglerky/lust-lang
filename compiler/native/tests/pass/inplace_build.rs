// Values built in their final place (release C, cgen's R1 and R3): a large
// return written through the out pointer as it is built, calls whose destination holds a value
// or is borrowed, return chains, loops, deferred and repeated initialization, and drop order.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Big {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
}

#[inline(never)]
fn make(n: u64) -> Big {
    Big { a: n, b: n + 1, c: n + 2, d: n + 3, e: n + 4 }
}

// Writes its result field by field while it still reads `x`: an out storage that `x` reaches
// would be clobbered before the reads.
#[inline(never)]
fn shift(x: &Big) -> Big {
    let mut r = Big { a: 0, b: 0, c: 0, d: 0, e: 0 };
    r.a = x.b;
    r.b = x.c;
    r.c = x.d;
    r.d = x.e;
    r.e = x.a;
    r
}

// Passes its own out storage on.
#[inline(never)]
fn chain(n: u64) -> Big {
    make(n * 10)
}

#[inline(never)]
fn twice(n: u64) -> Big {
    let first = make(n);
    shift(&first)
}

#[inline(never)]
fn sum(b: &Big) -> u64 {
    b.a + b.b * 3 + b.c * 5 + b.d * 7 + b.e * 11
}

struct Loud(u32, Vec<u32>, u64);

impl Drop for Loud {
    fn drop(&mut self) {
        println!("drop {} {:?} {}", self.0, self.1, self.2);
    }
}

#[inline(never)]
fn loud(n: u32) -> Loud {
    println!("make {n}");
    Loud(n, vec![n; 2], n as u64 * 100)
}

struct D(u32);

impl Drop for D {
    fn drop(&mut self) {
        println!("D {}", self.0);
    }
}

// Sol's case: the flag of `x` is clear on one path and set on the other at `x = D(2)`.
fn flags(b: bool) {
    let mut x = D(1);
    if b {
        drop(x);
    }
    x = D(2);
    drop(x);
}

fn main() {
    // The destination holds a value and the argument borrows it.
    let mut x = make(1);
    let r = &x;
    x = shift(r);
    println!("{x:?}");
    x = shift(&x);
    println!("{x:?}");
    // A fresh destination, and a chain of out pointers.
    let c = chain(7);
    println!("{c:?} {}", sum(&c));
    let t = twice(3);
    println!("{t:?}");
    // A borrowed local in a loop: each round's call writes it again.
    let mut acc = 0;
    for i in 0..4 {
        let b = make(i);
        acc += sum(&b);
        let s = shift(&b);
        acc ^= sum(&s);
    }
    println!("{acc}");
    // Into a field, and into an element.
    let mut pair = (make(2), 5u8);
    pair.0 = shift(&pair.0);
    let mut v = vec![make(4), make(5)];
    v[1] = shift(&v[0]);
    println!("{:?} {} {:?}", pair.0, pair.1, v[1]);
    // Drop glue: deferred initialization, then a value replaced.
    let l: Loud;
    l = Loud(5, vec![1, 2], 3);
    println!("built {}", l.0);
    let mut m = loud(1);
    m = loud(2);
    println!("have {}", m.0);
    let w = if acc > 3 { loud(3) } else { loud(4) };
    println!("chose {}", w.0);
    flags(false);
    flags(true);
    let moved = m;
    println!("moved {}", moved.0);
}
