// expect: L0306
// rustc E0597: an `iter_mut` item is `&mut &String`, invariant in the
// inner reference, so storing `&short` makes `short` outlive `v`.
fn main() {
    let long = String::from("long");
    let mut v: Vec<&String> = vec![&long];
    {
        let short = String::from("short");
        for slot in v.iter_mut() {
            *slot = &short;
        }
    }
    println!("{:?}", v);
}
