// io::ErrorKind of raw OS error codes (per-OS errno tables) and of a
// missing file.
fn main() {
    for code in [1, 2, 4, 5, 13, 22, 32] {
        let e = std::io::Error::from_raw_os_error(code);
        println!("{code} -> {:?}", e.kind());
    }
    let e = std::fs::read("/lust-no-such-dir-xyz/nope.txt").unwrap_err();
    println!("missing -> {:?} raw={:?}", e.kind(), e.raw_os_error());
}
