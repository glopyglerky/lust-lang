// Stores behind parameter references that keep borrowing from the same parameter.
fn dup_first(v: &mut Vec<&i64>) {
    let first = v[0];
    v.push(first);
}

fn reassign_same(mut a: &i64) -> i64 {
    a = &*a;
    *a
}

fn derived_local(v: &mut Vec<i64>) {
    let mut r = v;
    r = &mut Vec::new();
    println!("derived ok");
}

fn main() {
    let n: i64 = 9;
    let mut v: Vec<&i64> = vec![&n];
    dup_first(&mut v);
    println!("{} {}", v.len(), v[1]);
    println!("{}", reassign_same(&n));
    let mut w: Vec<i64> = Vec::new();
    derived_local(&mut w);
    println!("{}", w.len());
}
