// `?` inside closures returns from the closure; a captured Option or
// Result is consumed or borrowed as rustc's desugaring does.
fn main() {
    let base: Option<u32> = Some(10);
    let add = |x: u32| -> Option<u32> {
        let b = base?;
        Some(b + x)
    };
    println!("{:?} {:?}", add(1), base);
    let none: Option<u32> = None;
    let f = || -> Option<u32> { let v = none?; Some(v) };
    println!("{:?}", f());
    let parse = |s: u8| -> Result<u8, String> {
        if s > 9 { return Err(String::from("big")); }
        let r: Result<u8, String> = Ok(s);
        let v = r?;
        Ok(v * 2)
    };
    println!("{:?} {:?}", parse(3), parse(30));
}
