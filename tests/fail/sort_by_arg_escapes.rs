// expect: L0733
// rustc: borrowed data escapes outside of closure. sort_by's comparator
// takes higher-ranked `&T` arguments, so it cannot store one outside.
fn run(v: &mut Vec<i32>) {
    let mut keep = Vec::new();
    v.sort_by(|a, b| {
        keep.push(a);
        a.cmp(b)
    });
    println!("{}", keep.len());
}

fn main() {
    let mut v = vec![3, 1, 2];
    run(&mut v);
}
