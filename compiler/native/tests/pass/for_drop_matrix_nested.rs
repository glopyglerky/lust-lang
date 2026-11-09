// For-loop item drops in nested loops (SOL-COVERAGE-4 D2): the outer item
// lives across the inner loop; inner break and continue, outer continue after
// an inner loop, an inner loop over a borrow of the outer item, and a while
// loop around a for loop. `next` and every Drop print; bodies are call-free.
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop D({})", self.0);
    }
}
struct It(i32, i32);
impl Iterator for It {
    type Item = D;
    fn next(&mut self) -> Option<D> {
        if self.0 == 0 {
            return None;
        }
        self.0 -= 1;
        println!("  next {}.{}", self.1, self.0);
        Some(D(self.1 * 10 + self.0))
    }
}
impl Drop for It {
    fn drop(&mut self) {
        println!("  drop It({}.{})", self.1, self.0);
    }
}

fn main() {
    let mut k = 0;
    println!("plain nest");
    for a in It(2, 1) {
        for b in It(2, 2) {
            k += a.0 + b.0;
        }
    }
    println!("inner break, outer continue");
    for a in It(3, 3) {
        for b in It(3, 4) {
            if b.0 == 41 {
                break;
            }
            k += b.0;
        }
        if a.0 == 31 {
            continue;
        }
        k += a.0;
    }
    println!("inner continue");
    for _ in It(2, 5) {
        for b in It(3, 6) {
            if b.0 == 61 {
                continue;
            }
            k += 1;
        }
    }
    println!("inner over the outer item");
    for a in It(2, 7) {
        for b in It((a.0 % 3) + 1, 8) {
            k += b.0 - a.0;
        }
    }
    println!("while around for");
    let mut n = 0;
    while n < 2 {
        for x in It(2, 9) {
            k += x.0 + n;
        }
        n += 1;
    }
    println!("{k}");
}
