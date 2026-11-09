// Vec::drain: every range form, both ends, early drop, adapters,
// a leaked Drain, and drop order of the un-yielded rest.

struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn forms() {
    let mut v = vec![1, 2, 3, 4, 5];
    let mut d = v.drain(1..4);
    println!("len={} hint={:?} slice={:?}", d.len(), d.size_hint(), d.as_slice());
    println!("{:?} {:?} {:?}", d.next(), d.next_back(), d.as_slice());
    drop(d);
    println!("v={v:?}");
    let c: Vec<i32> = v.drain(..).collect();
    println!("all {c:?} {v:?}");
    let mut v = vec![1, 2, 3, 4, 5];
    let c: Vec<i32> = v.drain(2..).collect();
    println!("from {c:?} {v:?}");
    let c: Vec<i32> = v.drain(..1).collect();
    println!("to {c:?} {v:?}");
    let c: Vec<i32> = v.drain(..=0).collect();
    println!("toi {c:?} {v:?}");
    let mut v = vec![1, 2, 3, 4, 5];
    let c: Vec<i32> = v.drain(1..=3).collect();
    println!("incl {c:?} {v:?}");
    let r = 1..3;
    let c: Vec<i32> = v.drain(r).collect();
    println!("stored {c:?} {v:?}");
    let r = 0..=1;
    let mut v = vec![7, 8, 9];
    let c: Vec<i32> = v.drain(r).collect();
    println!("stored-incl {c:?} {v:?}");
    let mut v = vec![7, 8, 9];
    let c: Vec<i32> = v.drain((std::ops::Bound::Excluded(0), std::ops::Bound::Included(2))).collect();
    println!("bounds {c:?} {v:?}");
    let p = (std::ops::Bound::Unbounded, std::ops::Bound::Excluded(1usize));
    let mut v = vec![7, 8, 9];
    let c: Vec<i32> = v.drain(p).collect();
    println!("pair {c:?} {v:?}");
    // An empty drain at the end and at the start.
    let mut v = vec![1, 2, 3];
    println!("empty {} {:?}", v.drain(3..3).count(), v);
    println!("empty {} {:?}", v.drain(0..0).count(), v);
    // The two-phase borrow: the bounds read the receiver.
    let c: Vec<i32> = v.drain(v.len() - 2..).collect();
    println!("tp {c:?} {v:?}");
}

fn adapters() {
    let mut v = vec![1, 2, 3, 4, 5, 6];
    let c: Vec<i32> = v.drain(1..5).rev().map(|x| x * 10).collect();
    println!("adapt {c:?} {v:?}");
    let mut v = vec![1, 2, 3, 4];
    let mut d = v.drain(..);
    println!("nth {:?}", d.nth(2));
    println!("count {}", d.count());
    println!("after {v:?}");
    let s: i32 = vec![1, 2, 3].drain(..).sum();
    println!("sum {s}");
    let mut v = vec![10, 20, 30];
    for x in v.drain(1..) {
        print!("{x} ");
    }
    println!("| {v:?}");
}

fn drops_and_leak() {
    let mut v: Vec<D> = (0..6).map(D).collect();
    {
        let mut d = v.drain(1..5);
        match d.next() {
            Some(x) => println!("take {}", x.0),
            None => println!("take none"),
        }
        match d.next_back() {
            Some(x) => println!("take {}", x.0),
            None => println!("take none"),
        }
    }
    println!("left {}", v.len());
    drop(v);
    // A leaked Drain leaves the Vec at the drain start; the
    // un-yielded elements and the tail leak (no drops for 1, 2, 3).
    let mut v: Vec<D> = (0..4).map(D).collect();
    let d = v.drain(1..3);
    std::mem::forget(d);
    println!("forget len={}", v.len());
    drop(v);
    // Capacity is unchanged by a drain.
    let mut v: Vec<i32> = Vec::with_capacity(10);
    v.push(1); v.push(2); v.push(3); v.push(4);
    let c0 = v.capacity();
    for _ in v.drain(1..3) {}
    println!("cap {c0} {} {:?}", v.capacity(), v);
}

fn main() {
    forms();
    adapters();
    drops_and_leak();
}
