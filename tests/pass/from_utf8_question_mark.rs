// A FromUtf8Error converts into a user error through `?` and From.
use std::string::FromUtf8Error;
#[derive(Debug)]
enum MyErr {
    Utf8(FromUtf8Error),
}
impl From<FromUtf8Error> for MyErr {
    fn from(e: FromUtf8Error) -> MyErr {
        MyErr::Utf8(e)
    }
}
fn parse(v: Vec<u8>) -> Result<String, MyErr> {
    let s = String::from_utf8(v)?;
    Ok(s)
}
fn main() {
    println!("{:?}", parse(vec![104, 105]));
    println!("{:?}", parse(vec![0xc3]));
    if let Err(e) = String::from_utf8(vec![0xc3, 0x28]) {
        let msg = e.to_string();
        println!("{} {}", msg, msg.len());
    }
}
