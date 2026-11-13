// for over &Box<[T]> and &mut Box<[T]> (alloc's IntoIterator impls since 1.80).
#[derive(Debug, Clone)]
struct I {
    n: u32,
}
fn main() {
    let mut b: Box<[I]> = vec![I { n: 1 }, I { n: 2 }].into_boxed_slice();
    for x in &mut b {
        x.n += 10;
    }
    for x in &b {
        print!("{} ", x.n);
    }
    let r = &mut b;
    for x in r {
        x.n += 1;
    }
    let mut keep = Vec::new();
    for x in &b {
        keep.push(x);
    }
    println!("{:?}", keep);
}
