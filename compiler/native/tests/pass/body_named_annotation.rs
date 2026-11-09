// rustc accepts: a `let` annotation with an explicit lifetime (W-BORROW keeps its regions).
fn first<'a>(x: &'a [i32]) -> i32 {
    let y: &'a i32 = &x[0];
    *y
}

fn main() {
    println!("{}", first(&[3, 4]));
}
