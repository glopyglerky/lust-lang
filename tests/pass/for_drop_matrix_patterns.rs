// For-loop item drops by pattern (SOL-COVERAGE-4 D2): `next` and every Drop
// print, so the output is the full order of pulls and drops. Each pattern
// runs with a call-free body (only integer work) and with a calling body.
#![allow(unused_variables)]
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop D({})", self.0);
    }
}
struct S {
    a: D,
    b: D,
}
struct It(i32);
impl Iterator for It {
    type Item = (D, (D, D));
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        self.0 -= 1;
        println!("  next {}", self.0);
        let n = self.0 * 10;
        Some((D(n), (D(n + 1), D(n + 2))))
    }
}
impl Drop for It {
    fn drop(&mut self) {
        println!("  drop It({})", self.0);
    }
}
struct SIt(i32);
impl Iterator for SIt {
    type Item = S;
    fn next(&mut self) -> Option<S> {
        if self.0 == 0 {
            return None;
        }
        self.0 -= 1;
        println!("  next S {}", self.0);
        Some(S { a: D(self.0 * 10 + 5), b: D(self.0 * 10 + 6) })
    }
}

#[inline(never)]
fn note(n: i32) {
    println!("  note {n}");
}

fn main() {
    let mut k = 0;
    println!("wild, no call");
    for _ in It(2) {
        k += 1;
    }
    println!("wild, call");
    for _ in It(2) {
        note(k);
    }
    println!("name, no call");
    for x in It(2) {
        k += x.0.0;
    }
    println!("name, call");
    for x in It(2) {
        note(x.0.0);
    }
    println!("underscore name, no call");
    for _x in It(2) {
        k += 1;
    }
    println!("partial, no call");
    for (a, _) in It(2) {
        k += a.0;
    }
    println!("partial, call");
    for (a, _) in It(2) {
        note(a.0);
    }
    println!("nested partial, no call");
    for (_, (b, _)) in It(2) {
        k += b.0;
    }
    println!("all bound, no call");
    for (a, (b, c)) in It(2) {
        k += a.0 + b.0 + c.0;
    }
    println!("all bound, call");
    for (a, (b, c)) in It(2) {
        note(c.0);
    }
    println!("ref, no call");
    for ref x in It(2) {
        k += x.1.0.0;
    }
    println!("ref mut field, call");
    for (ref mut a, _) in It(2) {
        a.0 += 100;
        note(a.0);
    }
    println!("struct rest, no call");
    for S { a, .. } in SIt(2) {
        k += a.0;
    }
    println!("struct both, call");
    for S { b, a } in SIt(2) {
        note(b.0);
    }
    println!("{k}");
}
