// panic: wanted an error: 3
fn main() {
    let r: Result<u8, String> = Ok(3);
    let e = r.expect_err("wanted an error");
    println!("{}", e);
}
