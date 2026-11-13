// expect: L0303
fn main() {
    let mut v = vec![String::from("a")];
    let lens = v.iter().map(|s| s.len());
    v.clear();
    println!("{}", lens.sum::<usize>());
}
