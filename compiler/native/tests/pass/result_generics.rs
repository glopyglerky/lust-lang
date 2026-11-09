// Result in struct fields, Vec<Result>, generic fns over Result,
// while let, nested Option<Result>, ref patterns, or-patterns.
struct Wrap { r: Result<String, Vec<i32>> }
fn pick<T>(x: Result<T, String>, d: T) -> T {
    match x { Ok(v) => v, Err(_) => d }
}
fn main() {
    let w = Wrap { r: Err(vec![1, 2, 3]) };
    match w.r {
        Ok(s) => println!("ok {}", s),
        Err(v) => println!("err {:?} {}", v, v.len()),
    }
    let vs = vec![Ok(1), Err("a".to_string()), Ok(3)];
    println!("{:?}", vs);
    println!("{}", pick(Ok(5), 9));
    println!("{}", pick(Err("x".to_string()), 9));
    let mut it: Result<i32, String> = Ok(0);
    while let Ok(n) = it {
        println!("n={}", n);
        if n >= 2 { it = Err("done".to_string()); } else { it = Ok(n + 1); }
    }
    let o: Option<Result<i32, String>> = Some(Err("deep".to_string()));
    match o {
        Some(Ok(v)) => println!("so {}", v),
        Some(Err(e)) => println!("se {}", e),
        None => println!("none"),
    }
    let r: &Result<String, i32> = &Ok("hi".to_string());
    match r {
        Ok(s) => println!("ref {}", s),
        Err(n) => println!("ref {}", n),
    }
    let a: Result<i32, String> = Ok(1);
    let msg = match a { Ok(1) | Err(_) => "one-or-err", Ok(_) => "other" };
    println!("{}", msg);
}
