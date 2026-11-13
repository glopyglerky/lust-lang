// Iterating a Vec by value: elements move out in order, and the
// not-yet-yielded rest drops on early return and break.
struct G {
    n: i32,
}
impl Drop for G {
    fn drop(&mut self) {
        println!("drop {}", self.n);
    }
}
fn take2(v: Vec<G>) -> i32 {
    let mut seen = 0;
    for g in v {
        seen += 1;
        println!("see {}", g.n);
        if seen == 2 {
            return seen;
        }
    }
    seen
}
fn first_break(v: Vec<String>) {
    for s in v {
        println!("got {}", s);
        break;
    }
    println!("broke");
}
fn main() {
    let v = vec![G { n: 1 }, G { n: 2 }, G { n: 3 }, G { n: 4 }];
    println!("got {}", take2(v));
    first_break(vec!["a".to_string(), "bb".to_string(), "ccc".to_string()]);
    for n in vec![10, 20, 30] {
        println!("n {}", n);
    }
    let mut total = 0;
    for s in vec!["x".to_string(), "yy".to_string()] {
        total += s.len();
    }
    println!("total {}", total);
    let mut units: Vec<()> = Vec::new();
    units.resize(3, ());
    for _u in units {
        total += 1;
    }
    println!("units {}", total);
}
