// `&|n| n - 1`, a closure with no captures, is promoted to a `'static`
// constant, as rustc does, so the vec may hold it past the statement.
fn main() {
    let g = |n: i32| n * 2;
    let fs: Vec<&dyn Fn(i32) -> i32> = vec![&g, &|n| n - 1];
    println!("{}", fs.iter().map(|f| f(5)).sum::<i32>());
}
