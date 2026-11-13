// Width and precision of io::Error Display per representation
// (fz2-platform): a SimpleMessage error pads like std's, while the
// Simple (kind text) and Os forms ignore the flags, as in rustc.
fn main() {
    let sm = std::fs::write("a\0b", "x").unwrap_err();
    println!("[{:>50}] [{:.4}] [{:<50}] [{:^49}]", sm, sm, sm, sm);
    let simple = std::io::Error::from(std::io::ErrorKind::NotFound);
    println!("[{:>30}] [{:.4}]", simple, simple);
    let os = std::fs::read("/no/such/lust-test-file").unwrap_err();
    println!("[{:>60}] [{:.5}]", os, os);
    let custom = std::io::Error::new(std::io::ErrorKind::Other, "my text");
    println!("[{:>20}] [{:.3}]", custom, custom);
    let utf8 = std::fs::read_to_string("/bin/sh").unwrap_err();
    println!("[{:>55}] [{:.6}] {}", utf8, utf8, utf8.kind() == std::io::ErrorKind::InvalidData);
}
