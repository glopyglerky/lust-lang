// panic: called `Result::unwrap()` on an `Err` value: E { code: -1 }
#[derive(Debug)]
struct E {
    code: i32,
}
fn main() {
    let r: Result<String, E> = Err(E { code: -1 });
    println!("before");
    let s = r.unwrap();
    println!("{}", s);
}
