// std::io::Error and ErrorKind (W-SHIM): Display and Debug per
// representation, kinds of OS codes, conversions, and ordering.
use std::io::{self, ErrorKind};

fn describe(e: &io::Error) {
    println!("[{}] [{:?}] {:?} {:?}", e, e, e.kind(), e.raw_os_error());
}

fn open_fail(code: i32) -> io::Result<u32> {
    if code == 0 { Ok(7) } else { Err(io::Error::from_raw_os_error(code)) }
}

fn chain(code: i32) -> io::Result<u32> {
    let n = open_fail(code)?;
    Ok(n + 1)
}

fn main() {
    for code in [1, 2, 4, 11, 13, 17, 20, 21, 22, 28, 32, 39, 40, 95, 104, 110, 111, 9999] {
        describe(&io::Error::from_raw_os_error(code));
    }
    let nf: io::Error = ErrorKind::NotFound.into();
    describe(&nf);
    let k: io::Error = ErrorKind::UnexpectedEof.into();
    describe(&k);
    describe(&io::Error::other("boom"));
    describe(&io::Error::new(ErrorKind::InvalidData, String::from("bad\tdata \"q\"")));
    let e = io::Error::other("pad");
    println!("[{:>8}] [{:<8}] [{:^9}]", e, e, e);
    let e: io::Error = ErrorKind::PermissionDenied.into();
    println!("[{:>30}]", e);
    println!("{:#?}", io::Error::from_raw_os_error(2));
    println!("{:#?}", io::Error::other("multi"));
    let other: io::Error = ErrorKind::Other.into();
    println!("{:#?}", other);
    let kinds = [ErrorKind::NotFound, ErrorKind::Other, ErrorKind::BrokenPipe, ErrorKind::WouldBlock, ErrorKind::Interrupted];
    for k in kinds {
        println!("{} | {:?} | [{:>12}] | {}", k, k, k, k == ErrorKind::Other);
    }
    println!("{} {}", ErrorKind::NotFound < ErrorKind::Other, ErrorKind::Unsupported > ErrorKind::Interrupted);
    println!("{:?}", ErrorKind::OutOfMemory.cmp(&ErrorKind::NotFound));
    match chain(2) {
        Ok(n) => println!("ok {n}"),
        Err(e) if e.kind() == ErrorKind::NotFound => println!("missing: {e}"),
        Err(e) => println!("other: {e}"),
    }
    match chain(0) {
        Ok(n) => println!("ok {n}"),
        Err(e) => println!("err {e}"),
    }
    let s = io::Error::from_raw_os_error(13).to_string();
    println!("{} {}", s.len(), s);
    let r: io::Result<()> = Err(io::Error::other(String::from("owned")));
    println!("{:?}", r);
}
