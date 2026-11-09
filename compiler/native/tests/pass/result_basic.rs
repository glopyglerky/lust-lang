// Result<T, E>: constructors, match, if let, Debug, Eq, Clone, Copy,
// qualified and turbofish spellings, size_of (niche layout).
fn div(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 { Err("zero".to_string()) } else { Ok(a / b) }
}
fn main() {
    let x = div(10, 2);
    let y = div(1, 0);
    println!("{:?} {:?}", x, y);
    match x {
        Ok(v) => println!("ok {}", v),
        Err(e) => println!("err {}", e),
    }
    if let Err(e) = y {
        println!("caught {}", e);
    }
    let z: Result<i32, String> = Ok(7);
    println!("{} {} {}", z == Ok(7), z == Err("q".to_string()), div(3, 0) == div(4, 0));
    let w = z.clone();
    println!("{:?}", w);
    let c: Result<i32, i32> = Err(3);
    let c2 = c;
    println!("{:?} {:?} {}", c, c2, std::mem::size_of::<Result<&i32, ()>>());
    println!("{}", std::mem::size_of::<Result<u8, u64>>());
    println!("{}", std::mem::size_of::<Result<String, Vec<i32>>>());
    let nested: Result<Option<i32>, String> = Ok(Some(4));
    println!("{:?}", nested);
    let q = Result::<i32, String>::Ok(9);
    println!("{:?}", q);
}
