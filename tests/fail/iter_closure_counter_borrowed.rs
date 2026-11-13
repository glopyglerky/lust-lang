// expect: L0303
fn main() {
    let v = vec![1, 2, 3];
    let mut k = 0;
    let it = v.iter().map(|x| {
        k += 1;
        *x
    });
    println!("{}", k);
    let s: i32 = it.sum();
    println!("{}", s);
}
