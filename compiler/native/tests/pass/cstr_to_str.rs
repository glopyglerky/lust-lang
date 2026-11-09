// CStr::to_str: str::from_utf8 of the bytes before the NUL.
use std::ffi::CStr;
fn main() {
    let a = CStr::from_bytes_with_nul(b"hi\0").unwrap();
    println!("{:?} {:?}", a.to_str(), a.to_str().is_ok());
    let b = CStr::from_bytes_with_nul(b"h\xffi\0").unwrap();
    let e = b.to_str().unwrap_err();
    println!("{:?} {} {} {:?}", e, e, e.valid_up_to(), e.error_len());
    let c = CStr::from_bytes_with_nul(b"\0").unwrap();
    println!("{:?}", c.to_str());
}
