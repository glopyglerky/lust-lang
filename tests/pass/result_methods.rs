// Result methods without closures: is_ok, is_err, ok, err, unwrap_or,
// unwrap, expect, unwrap_err, expect_err.
#[derive(Debug)]
struct E {
    code: i32,
}
fn get(n: i32) -> Result<String, E> {
    if n > 0 { Ok(format!("v{}", n)) } else { Err(E { code: n }) }
}
fn main() {
    let a = get(1);
    let b = get(-2);
    println!("{} {} {} {}", a.is_ok(), a.is_err(), b.is_ok(), b.is_err());
    println!("{:?} {:?}", get(3).ok(), get(-3).ok());
    println!("{:?} {:?}", get(4).err(), get(-4).err());
    println!("{} {}", get(5).unwrap_or(String::from("d")), get(-5).unwrap_or(String::from("d")));
    println!("{} {}", get(6).unwrap(), get(7).expect("seven"));
    println!("{:?} {:?}", get(-8).unwrap_err(), get(-9).expect_err("nine"));
}
