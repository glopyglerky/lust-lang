// For-loop item drops on control flow exits (SOL-COVERAGE-4 D2): continue,
// break, labeled break, return, `?`, a moved item, a conditionally moved item,
// and a borrowed iterator that keeps going after a break. `next` and every
// Drop print; bodies are call-free where the case allows it.
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

fn ret_early(n: i32) -> i32 {
    let mut k = 0;
    for x in It(4) {
        if x.0 == n {
            return k;
        }
        k += 1;
    }
    -1
}

fn question(n: i32) -> Option<i32> {
    let mut k = 0;
    for x in It(4) {
        let v = if x.0 == n { None } else { Some(x.0) };
        k += v?;
    }
    Some(k)
}

fn main() {
    let mut k = 0;
    println!("continue, no call");
    for x in It(3) {
        if x.0 == 1 {
            continue;
        }
        k += x.0;
    }
    println!("continue before the binding is used");
    for _ in It(3) {
        k += 1;
        if k % 2 == 0 {
            continue;
        }
        k += 10;
    }
    println!("break, no call");
    for x in It(4) {
        if x.0 == 2 {
            break;
        }
        k += x.0;
    }
    println!("labeled break from the inner loop");
    'outer: for a in It(2) {
        for b in It(3) {
            if b.0 == 1 {
                break 'outer;
            }
            k += a.0 + b.0;
        }
    }
    println!("return from inside: {}", ret_early(2));
    println!("question mark: {:?}", question(1));
    println!("question mark, no exit: {:?}", question(9));
    println!("moved into a vec");
    let mut keep = Vec::new();
    for x in It(3) {
        keep.push(x);
    }
    println!("  kept {}", keep.len());
    drop(keep);
    println!("moved on one path");
    let mut some = Vec::new();
    for x in It(4) {
        if x.0 % 2 == 0 {
            some.push(x);
        }
    }
    println!("  kept {}", some.len());
    drop(some);
    println!("by_ref, break, then the rest");
    let mut it = It(5);
    for x in it.by_ref() {
        if x.0 == 3 {
            break;
        }
    }
    for x in it.by_ref() {
        k += x.0;
    }
    drop(it);
    println!("{k}");
}
