// `for_each`'s closure argument is an item (free): storing it in a
// captured Vec keeps the borrow of `v`, not of `out`.
fn main() {
    let v = vec![1, 2, 3];
    let mut out: Vec<&i32> = Vec::new();
    v.iter().for_each(|x| out.push(x));
    println!("{}", out.len());
}
