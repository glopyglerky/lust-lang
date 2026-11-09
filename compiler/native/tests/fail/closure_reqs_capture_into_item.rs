// expect: L0306
// rustc E0597: the body stores the captured `&short` through an
// `iter_mut` item, so `short` must outlive `v`'s elements.
fn main() {
    let long = String::from("long");
    let mut v: Vec<&String> = vec![&long];
    {
        let short = String::from("short");
        v.iter_mut().for_each(|slot| *slot = &short);
    }
    println!("{:?}", v);
}
