// smallvec 1.15.1 drop and clone order: truncate and clear drop from
// the back, retain and dedup drop after every call, resize clones then
// drops its value, and from_elem clones as upstream does.
use smallvec::SmallVec;

// The log is stdout itself: each clone and drop prints at once, and
// `flush` ends the line with the step's name.
fn flush(what: &str) {
    println!("| {}", what);
}

#[derive(Debug, PartialEq)]
struct D(u8);

impl Clone for D {
    fn clone(&self) -> D {
        print!("c{} ", self.0);
        D(self.0)
    }
}

impl Drop for D {
    fn drop(&mut self) {
        print!("d{} ", self.0);
    }
}

fn main() {
    let mut v: SmallVec<[D; 3]> = SmallVec::new();
    for i in 0..6u8 {
        v.push(D(i));
    }
    flush("push");
    v.truncate(3);
    flush("truncate");
    v.retain(|d| d.0 != 1);
    flush("retain");
    v.resize(5, D(9));
    flush("resize");
    v.push(D(9));
    v.dedup();
    flush("dedup");
    let w: SmallVec<[D; 2]> = SmallVec::from_elem(D(4), 2);
    flush("from_elem inline");
    let x: SmallVec<[D; 2]> = SmallVec::from_elem(D(5), 3);
    flush("from_elem heap");
    let c = v.clone();
    flush("clone");
    v.clear();
    flush("clear");
    drop(c);
    flush("drop clone");
    drop(w);
    drop(x);
    flush("drop rest");
    // drain: items not yielded drop front to back when the Drain drops.
    let mut y: SmallVec<[D; 2]> = SmallVec::new();
    for i in 0..5u8 {
        y.push(D(i));
    }
    flush("push five");
    let mut d = y.drain(1..4);
    let first = d.next();
    flush("drain next");
    drop(d);
    flush("drain drop");
    drop(first);
    flush("drop first");
    println!("{} {}", y.len(), y.capacity());
    drop(y);
    flush("drop drained");
}
