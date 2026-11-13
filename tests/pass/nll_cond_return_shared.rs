// A conditional return of a shared borrow does not block later shared borrows.
fn first_or_second(v: &Vec<i32>) -> &i32 {
    if let Some(x) = v.first() {
        if *x > 10 {
            return x;
        }
    }
    &v[1]
}

fn main() {
    let v = vec![20, 3];
    println!("{}", first_or_second(&v));
    let w = vec![1, 4];
    println!("{}", first_or_second(&w));
}
