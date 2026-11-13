// W-BORROW3: an explicit `for<'a>` binder that means what elision means.
fn apply(f: for<'a> fn(&'a i64) -> i64) -> i64 {
    f(&2)
}

fn inc(x: &i64) -> i64 {
    x + 1
}

fn main() {
    println!("{}", apply(inc));
}
