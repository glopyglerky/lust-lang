// The ? operator error conversion: user From impl, generic errors,
// numeric widening From, closures.
#[derive(Debug)]
struct MyErr { code: i32 }
struct Wrap { msg: String }
impl From<MyErr> for Wrap {
    fn from(e: MyErr) -> Wrap { Wrap { msg: format!("wrapped {}", e.code) } }
}
fn inner(x: i32) -> Result<i32, MyErr> { if x < 0 { Err(MyErr { code: -x }) } else { Ok(x) } }
fn outer(x: i32) -> Result<i32, Wrap> {
    let v = inner(x)?;
    Ok(v + 1)
}
fn num(x: i32) -> Result<i32, i32> { if x < 0 { Err(x) } else { Ok(x) } }
fn widen(x: i32) -> Result<i32, i64> {
    let v = num(x)?;
    Ok(v)
}
fn str_err() -> Result<i32, String> {
    let r: Result<i32, &str> = Err("oops");
    Ok(r? + 1)
}
fn ref_string_err(s: &String) -> Result<i32, String> {
    let r: Result<i32, &String> = Err(s);
    Ok(r?)
}
fn conv<E1, E2: From<E1>>(r: Result<i32, E1>) -> Result<i32, E2> { Ok(r? + 1) }
fn main() {
    match outer(-3) { Ok(v) => println!("ok {}", v), Err(w) => println!("err {}", w.msg) }
    match widen(-2) { Ok(v) => println!("ok {}", v), Err(e) => println!("err {}", e) }
    println!("{:?}", str_err());
    println!("{:?}", ref_string_err(&"owned".to_string()));
    match conv::<MyErr, Wrap>(Err(MyErr { code: 5 })) { Ok(v) => println!("ok {}", v), Err(w) => println!("err {}", w.msg) }
    match conv::<i32, i64>(Err(-4)) { Ok(v) => println!("ok {}", v), Err(e) => println!("err {}", e) }
}

