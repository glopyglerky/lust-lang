// expect: L0612
// unwrap shows the error with Debug.
struct NoDebug;
fn main() {
    let r: Result<u8, NoDebug> = Ok(1);
    println!("{}", r.unwrap());
}
