// Closures return and match on Result values; a captured Result is
// read through its variants.
fn main() {
    let limit = 10u32;
    let check = |x: u32| -> Result<u32, String> {
        if x > limit { Err(format!("{} > {}", x, limit)) } else { Ok(x * 2) }
    };
    println!("{:?} {:?}", check(3), check(30));
    let r: Result<u8, u8> = Ok(4);
    let get = || match r { Ok(v) => v, Err(e) => e + 100 };
    println!("{} {:?}", get(), r);
}
