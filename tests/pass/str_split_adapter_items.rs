// Adapters that store or pair split items (peekable, enumerate, zip,
// chain): the item is `&str` in the generated C too (W-STD-B).
fn main() {
    let mut it = "x,y,z".split(',').peekable();
    let first = match it.peek() {
        Some(s) => s.len(),
        None => 0,
    };
    println!("{} {:?}", first, it.next());
    let v: Vec<(usize, &str)> = "a b c".split(' ').enumerate().collect();
    println!("{:?}", v);
    let z: Vec<(&str, &str)> = "1,2".split(',').zip("x y".split_whitespace()).collect();
    println!("{:?}", z);
    let mut l = "p\nq\nr".lines().peekable();
    while let Some(x) = l.next() {
        if let Some(n) = l.peek() {
            println!("{x} then {n}");
        }
    }
    let c: Vec<&str> = "a-b".split('-').chain("c d".split(' ')).collect();
    println!("{:?} {:?}", c, "k=v".split_once('='));
}
