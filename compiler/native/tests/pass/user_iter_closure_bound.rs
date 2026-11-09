// User iterators whose Iterator impl carries a closure bound, through
// adapters: mono binds the impl's params from Self and from the bound's
// callable (a closure or a fn pointer), so `collect` and `chain` step them.
struct FromFn<F>(F);
impl<T, F> Iterator for FromFn<F>
where
    F: FnMut() -> Option<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        (self.0)()
    }
}

struct Succ<T, F> {
    next: Option<T>,
    f: F,
}
impl<T, F> Iterator for Succ<T, F>
where
    F: FnMut(&T) -> Option<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let x = self.next.take()?;
        self.next = (self.f)(&x);
        Some(x)
    }
}

struct Counter<F> {
    n: u32,
    f: F,
}
impl<F: Fn(u32) -> bool> Iterator for Counter<F> {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.n += 1;
        if (self.f)(self.n) { Some(self.n) } else { None }
    }
}

fn none() -> Option<i32> {
    None
}

fn main() {
    let mut c = 0;
    let v: Vec<i32> = FromFn(|| {
        c += 1;
        if c < 4 { Some(c) } else { None }
    })
    .collect();
    println!("{:?}", v);
    let w: Vec<i32> = FromFn(none as fn() -> Option<i32>).chain(vec![7]).collect();
    println!("{:?}", w);
    let p: Vec<u32> = Succ { next: Some(1u32), f: |&x: &u32| x.checked_mul(10) }.take(4).collect();
    println!("{:?}", p);
    let lim = 3;
    println!("{:?}", Counter { n: 0, f: move |n| n < lim }.map(|n| n * 2).collect::<Vec<_>>());
}
