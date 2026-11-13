// `for x in v` over a Vec by value iterates its vec::IntoIter: the item drops at the end of
// each iteration (or where it moves), and the elements not yet taken drop front to back when the
// loop exits by break, labeled break, continue to the end, early return or `?`. ZST elements,
// Strings moved out per element, nested loops over the same Vec type, an empty Vec and a Vec
// made by a call.
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

fn ds(n: i32) -> Vec<D> {
    (0..n).map(D).collect()
}

fn ret_early(v: Vec<D>, stop: i32) -> i32 {
    let _guard = D(100);
    for x in v {
        if x.0 == stop {
            println!("  return at {}", x.0);
            return x.0;
        }
        println!("  item {}", x.0);
    }
    -1
}

fn question(v: Vec<D>, bad: i32) -> Option<i32> {
    let mut k = 0;
    for x in v {
        let r = if x.0 == bad { None } else { Some(x.0) };
        k += r?;
        println!("  sum {k}");
    }
    Some(k)
}

fn check(s: &str) -> Result<usize, String> {
    if s.starts_with('!') { Err(format!("bad {s}")) } else { Ok(s.len()) }
}

fn result_question(v: Vec<String>) -> Result<usize, String> {
    let mut n = 0;
    for s in v {
        n += check(&s)?;
    }
    Ok(n)
}

fn main() {
    println!("break");
    for x in ds(5) {
        if x.0 == 2 {
            println!("  break at {}", x.0);
            break;
        }
        println!("  item {}", x.0);
    }
    println!("after break");

    println!("continue to the end");
    for x in ds(3) {
        if x.0 == 1 {
            continue;
        }
        println!("  item {}", x.0);
    }

    println!("item moved out, then break");
    let mut kept = Vec::new();
    for x in ds(4) {
        if x.0 == 1 {
            kept.push(x);
            continue;
        }
        if x.0 == 2 {
            break;
        }
    }
    println!("kept {}", kept.len());
    drop(kept);

    println!("early return");
    let r = ret_early(ds(4), 1);
    println!("returned {r}");
    println!("no early return");
    let r = ret_early(ds(2), 9);
    println!("returned {r}");

    println!("question mark");
    println!("{:?}", question(ds(4), 2));
    println!("{:?}", question(ds(3), 9));
    println!("{:?}", result_question(vec!["ab".to_string(), "!no".to_string(), "cd".to_string()]));
    println!("{:?}", result_question(vec!["ab".to_string(), "cde".to_string()]));

    println!("zst elements");
    let zs = vec![Z, Z, Z, Z];
    let mut taken = 0;
    for z in zs {
        taken += 1;
        if taken == 2 {
            drop(z);
            println!("  break after {taken}");
            break;
        }
    }
    let units = vec![(); 7];
    let mut n = 0;
    for () in units {
        n += 1;
    }
    println!("units {n}");

    println!("strings moved out per element");
    let words: Vec<String> = ["alpha", "beta", "gamma", "delta"].iter().map(|s| s.to_string()).collect();
    let mut out = Vec::new();
    for w in words {
        if w == "gamma" {
            break;
        }
        out.push(w);
    }
    println!("{out:?}");
    let mut total = String::new();
    for w in out {
        total.push_str(&w);
        total.push('/');
    }
    println!("{total}");

    println!("nested loops over the same Vec type");
    let grid: Vec<Vec<D>> = vec![ds(3), (10..13).map(D).collect(), (20..22).map(D).collect()];
    'outer: for row in grid {
        println!("  row of {}", row.len());
        for x in row {
            if x.0 == 11 {
                println!("  break outer at {}", x.0);
                break 'outer;
            }
            if x.0 == 1 {
                println!("  break inner at {}", x.0);
                break;
            }
            println!("  item {}", x.0);
        }
    }

    println!("empty vec");
    for x in Vec::<D>::new() {
        println!("  never {}", x.0);
    }
    println!("end");
}
