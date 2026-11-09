// Locals with alignment over 16 (fx-overalign): release C declares them plainly (the C
// compiler realigns the frame), dev C behind an aligned buffer. Values of 32-, 64- and
// 128-byte alignment are built, copied, returned, pushed, wrapped and captured, and every
// local's address must stay a multiple of its alignment.

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(align(32))]
struct S32 {
    a: u8,
    b: u64,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(align(64))]
struct S64 {
    id: u32,
    name: String,
    tags: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(align(128))]
struct S128 {
    x: [u64; 5],
    y: u8,
}

fn aligned<T>(x: &T) -> bool {
    (x as *const T as usize) % std::mem::align_of::<T>() == 0
}

#[inline(never)]
fn make64(id: u32) -> S64 {
    let s = S64 { id, name: format!("n{id}"), tags: vec![id as u16; 3] };
    assert!(aligned(&s));
    let t = s;
    assert!(aligned(&t));
    t
}

#[inline(never)]
fn make128(k: u64) -> S128 {
    let mut v = S128 { x: [k; 5], y: k as u8 };
    v.x[2] += 1;
    let w = v;
    assert!(aligned(&v) && aligned(&w));
    w
}

#[inline(never)]
fn pick(a: S32, b: S32, first: bool) -> S32 {
    let r = if first { a } else { b };
    assert!(aligned(&r));
    r
}

fn main() {
    let a = S32 { a: 1, b: 2 };
    let b = S32 { a: 3, b: 4 };
    let c = pick(a, b, false);
    println!("{:?} {}", c, aligned(&c));

    let mut v: Vec<S64> = Vec::new();
    for i in 0..5 {
        let s = make64(i);
        let o = Some(s);
        if let Some(s) = o {
            assert!(aligned(&s));
            v.push(s);
        }
    }
    println!("{} {} {:?}", v.len(), v.iter().all(aligned), v[4]);

    let built = || S128 { x: [7; 5], y: 9 };
    let m = built();
    let n = make128(m.x[0]);
    let both = (m, n);
    assert!(aligned(&both.0) && aligned(&both.1));
    println!("{:?} {}", both.1, aligned(&both));

    let swapped = {
        let mut p = make64(10);
        let mut q = make64(11);
        std::mem::swap(&mut p, &mut q);
        assert!(aligned(&p) && aligned(&q));
        (p.id, q.id)
    };
    println!("{:?}", swapped);

    let r: Result<S128, S64> = if v.len() > 3 { Ok(make128(3)) } else { Err(make64(0)) };
    match r {
        Ok(x) => println!("ok {} {}", x.x[2], aligned(&x)),
        Err(e) => println!("err {}", e.id),
    }
    let boxed: Vec<Box<S128>> = (0..3).map(|k| Box::new(make128(k))).collect();
    println!("{} {}", boxed.iter().map(|b| b.x[2]).sum::<u64>(), boxed.iter().all(|b| aligned(&**b)));
}
