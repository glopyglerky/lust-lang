// Later patterns may borrow other slots while earlier ref mut bindings live.
fn main() {
    let mut arr = [String::new(), String::new()];
    let [ref mut a, ref mut b] = arr;
    a.push('x'); b.push('y');
    let [_, ref mut c] = arr;
    let [ref e, _] = arr;
    c.push('z');
    println!("{} {:?}", e, arr);
}
