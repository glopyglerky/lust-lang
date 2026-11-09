// expect: L0306
// rustc E0597: `map(id)` yields `&u8`s borrowed from `w` (`id` returns its
// input), so the items may not outlive `w`. A fn item, a fn pointer, or a
// `dyn Fn` as the callable, not only a closure.
fn id(x: &u8) -> &u8 { x }
fn main() {
    let r: Vec<&u8>;
    {
        let w = vec![9u8, 3];
        r = w.iter().map(id).collect();
    }
    println!("{:?}", r);
}
