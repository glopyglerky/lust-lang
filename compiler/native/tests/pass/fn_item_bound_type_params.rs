// A fn item, a fn pointer, or a type param at `F: Fn(T) -> R`: `R` may
// borrow what `T` borrows, and nothing else.
fn keep<T, R, F: Fn(T) -> R>(f: F, x: T) -> R {
    f(x)
}

fn outer<'a, F: Fn(&u8) -> &u8>(f: F, d: &'a u8) -> &'a u8 {
    keep(f, d)
}

fn len(v: &Vec<u8>) -> usize {
    v.len()
}

fn first(v: &[u8]) -> Option<&u8> {
    v.first()
}

fn main() {
    let k = 3u8;
    println!("{}", outer(|x| x, &k));
    let r;
    {
        let v = vec![1u8, 2];
        r = keep(len, &v);
    }
    println!("{}", r);
    let data = vec![4u8, 5];
    let p: fn(&[u8]) -> Option<&u8> = first;
    println!("{:?} {:?}", keep(first, &data), keep(p, &data[1..]));
}
