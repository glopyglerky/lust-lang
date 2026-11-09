fn picks() -> Vec<i64> {
    let mut v: Vec<i64> = Vec::new();
    v.push(3);
    v.push(1);
    v.push(4);
    v
}

fn main() {
    let v = picks();
    let mut total = 0;
    let mut i = 0;
    while i < v.len() {
        total += v[i];
        i += 1;
    }
    println!("featured total {} of {}", total, v.len());
}
