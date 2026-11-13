// `dyn Fn` and `dyn FnMut` values as iterator adapter callables: behind a
// reference, a `&mut`, or a box, as a param or a local (the expansion calls
// the vtable's call entry). A `&mut dyn FnMut` passed to `fold` keeps the
// iterator's borrow out of its object lifetime.
fn count(v: &[u8], f: &dyn Fn(&u8) -> bool) -> usize {
    v.iter().copied().filter(f).count()
}

fn total(v: &[u8], f: &mut dyn FnMut(u32, u8) -> u32) -> u32 {
    v.iter().copied().fold(0, f)
}

fn scaled(v: &[u8], f: Box<dyn Fn(u8) -> u8>) -> Vec<u8> {
    v.iter().copied().map(f).collect()
}

fn main() {
    let v = vec![1u8, 2, 3, 4];
    let k = 2u8;
    let big = |x: &u8| *x > k;
    println!("{}", count(&v, &big));
    let mut calls = 0u32;
    let mut sum = |a: u32, x: u8| {
        calls += 1;
        a * 3 + x as u32
    };
    println!("{}", total(&v, &mut sum));
    println!("{calls}");
    println!("{:?}", scaled(&v, Box::new(move |x| x * k)));
    let g: &dyn Fn(u8) -> bool = &|x| x % 2 == 0;
    println!("{} {}", v.iter().copied().any(g), v.iter().copied().all(g));
    let h: Box<dyn FnMut(u8) -> Option<u8>> = Box::new(|x| if x > 1 { Some(x + 10) } else { None });
    println!("{:?}", v.iter().copied().filter_map(h).collect::<Vec<u8>>());
    let first: &dyn Fn(&u8) -> bool = &|x| *x > 2;
    println!("{:?} {:?}", v.iter().copied().find(first), v.iter().position(|x| first(x)));
}
