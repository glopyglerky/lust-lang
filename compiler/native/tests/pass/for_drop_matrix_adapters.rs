// For-loop item drops through owned adapters and std iterators
// (SOL-COVERAGE-4 D2): map, filter, take, skip, enumerate, zip, chain,
// peekable, take_while, skip_while, step_by, flat_map, and Vec and array
// into_iter with a break (the rest drop with the iterator). `next` and every
// Drop print; the loop bodies make no call.
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop D({})", self.0);
    }
}
struct It(i32);
impl Iterator for It {
    type Item = D;
    fn next(&mut self) -> Option<D> {
        if self.0 == 0 {
            return None;
        }
        self.0 -= 1;
        println!("  next {}", self.0);
        Some(D(self.0))
    }
}
impl Drop for It {
    fn drop(&mut self) {
        println!("  drop It({})", self.0);
    }
}

fn main() {
    let mut k = 0;
    println!("map");
    for x in It(2).map(|d| D(d.0 + 100)) {
        k += x.0;
    }
    println!("filter");
    for x in It(4).filter(|d| d.0 % 2 == 0) {
        k += x.0;
    }
    println!("take");
    for _ in It(4).take(2) {
        k += 1;
    }
    println!("skip");
    for x in It(4).skip(2) {
        k += x.0;
    }
    println!("enumerate");
    for (i, x) in It(2).enumerate() {
        k += i as i32 + x.0;
    }
    println!("zip");
    for (a, _) in It(2).zip(It(3)) {
        k += a.0;
    }
    println!("chain");
    for x in It(1).chain(It(2)) {
        k += x.0;
    }
    println!("peekable");
    let mut p = It(3).peekable();
    while let Some(x) = p.next() {
        if p.peek().is_some() {
            k += x.0;
        }
    }
    drop(p);
    println!("take_while");
    for x in It(4).take_while(|d| d.0 > 1) {
        k += x.0;
    }
    println!("skip_while");
    for x in It(4).skip_while(|d| d.0 > 1) {
        k += x.0;
    }
    println!("step_by");
    for x in It(5).step_by(2) {
        k += x.0;
    }
    println!("flat_map");
    for x in It(2).flat_map(|d| It(d.0 + 1)) {
        k += x.0;
    }
    println!("vec into_iter, break");
    let v = vec![D(50), D(51), D(52), D(53)];
    for x in v {
        if x.0 == 51 {
            break;
        }
        k += x.0;
    }
    println!("array into_iter, continue");
    for x in [D(60), D(61), D(62)] {
        if x.0 == 61 {
            continue;
        }
        k += x.0;
    }
    println!("{k}");
}
