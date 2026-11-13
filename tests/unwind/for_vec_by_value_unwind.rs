// A panic in the body of `for x in v` over a Vec by value: the item drops, then the elements
// not yet taken drop front to back as the unwind leaves the loop, then the fn's own locals;
// catch_unwind returns the panic after. Nested loops, ZST elements, and a panic after some
// Strings were moved out.
use std::panic;

struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop D({})", self.0);
    }
}

struct Z;
impl Drop for Z {
    fn drop(&mut self) {
        println!("  drop Z");
    }
}

fn ds(range: std::ops::Range<i32>) -> Vec<D> {
    range.map(D).collect()
}

fn walk(v: Vec<D>, at: i32) -> i32 {
    let _guard = D(100);
    let mut sum = 0;
    for x in v {
        if x.0 == at {
            panic!("boom at {}", x.0);
        }
        sum += x.0;
    }
    sum
}

fn nested(grid: Vec<Vec<D>>, at: i32) {
    for row in grid {
        println!("  row of {}", row.len());
        for x in row {
            if x.0 == at {
                panic!("inner boom at {}", x.0);
            }
            println!("  item {}", x.0);
        }
    }
}

fn zsts(n: usize, at: usize) {
    let v: Vec<Z> = (0..n).map(|_| Z).collect();
    let mut i = 0;
    for z in v {
        i += 1;
        if i == at {
            panic!("zst boom at {i}");
        }
        drop(z);
    }
}

fn strings(at: &str) -> usize {
    let words: Vec<String> = ["a", "bb", "ccc", "dddd"].iter().map(|s| s.to_string()).collect();
    let mut kept = Vec::new();
    for w in words {
        if w == at {
            panic!("string boom at {w}, kept {}", kept.len());
        }
        kept.push(w);
    }
    kept.len()
}

fn main() {
    panic::set_hook(Box::new(|info| {
        let msg = info.payload().downcast_ref::<String>().map(|s| s.as_str()).unwrap_or("?");
        println!("panic: {msg}");
    }));
    println!("panic mid-loop");
    let r = panic::catch_unwind(|| walk(ds(0..5), 2));
    println!("caught {}", r.is_err());
    println!("no panic");
    let r = panic::catch_unwind(|| walk(ds(0..3), 9));
    println!("ok {:?}", r.ok());
    println!("panic in the inner of nested loops");
    let r = panic::catch_unwind(|| nested(vec![ds(0..3), ds(10..13), ds(20..22)], 11));
    println!("caught {}", r.is_err());
    println!("zst elements");
    let r = panic::catch_unwind(|| zsts(4, 2));
    println!("caught {}", r.is_err());
    println!("strings");
    let r = panic::catch_unwind(|| strings("ccc"));
    println!("caught {}", r.is_err());
    println!("end");
}
