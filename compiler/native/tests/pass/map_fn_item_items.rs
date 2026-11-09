// Items of `map` with a fn item, fn pointer, method path, or `dyn Fn` borrow
// what the callable's input borrows: they live as long as the source.
fn id(x: &u8) -> &u8 { x }
fn main() {
    let w = vec![9u8, 3];
    let f: fn(&u8) -> &u8 = |x| x;
    let a: Vec<&u8> = w.iter().map(f).collect();
    let b: Vec<&u8> = w.iter().map(id).collect();
    let d: &dyn Fn(&u8) -> &u8 = &|x| x;
    let c: Vec<&u8> = w.iter().map(d).collect();
    let s = vec![String::from("x")];
    let e: Vec<&str> = s.iter().map(String::as_str).collect();
    let mut g: Vec<&u8> = Vec::new();
    g.extend(w.iter().map(id));
    println!("{:?} {:?} {:?} {:?} {:?}", a, b, c, e, g);
    let mut m = std::collections::BTreeMap::new();
    m.insert(1u8, 2u8);
    println!("{:?}", m.iter().collect::<Vec<_>>());
}
