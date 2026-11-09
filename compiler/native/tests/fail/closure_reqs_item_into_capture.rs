// expect: L0306
// rustc E0597: the body pushes the item (a borrow of `v`) into `out`.
fn main() {
    let mut out: Vec<&String> = Vec::new();
    {
        let v = vec![String::from("a")];
        v.iter().for_each(|x| out.push(x));
    }
    println!("{:?}", out);
}
