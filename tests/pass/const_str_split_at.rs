// `str::split_at` in const items (ts-rust VERSION_MAJOR_MINOR), on ASCII
// and multibyte text, and the halves used at run time. (A run-time
// `split_at` call is a str API outside this test.)
const VERSION: &str = "1.22.3";
const VERSION_MAJOR_MINOR: &str = {
    let bytes = VERSION.as_bytes();
    let mut seen_major = false;
    let mut i = 0;
    loop {
        assert!(i < bytes.len(), "invalid version string");
        if bytes[i] == b'.' {
            if seen_major {
                break;
            }
            seen_major = true;
        }
        i += 1;
    }
    VERSION.split_at(i).0
};
const PARTS: (&str, &str) = "héllo".split_at(3);
static TAIL: &str = "日本語x".split_at(6).1;
const F: &str = "abc".split_at(3).0;
const E: &str = "abc".split_at(0).0;
fn main() {
    println!("{} {}", VERSION_MAJOR_MINOR, VERSION_MAJOR_MINOR.len());
    println!("{:?} {} {}", PARTS, PARTS.0.len(), PARTS.1.chars().count());
    println!("[{}] [{}] [{}] {}", TAIL, F, E, E.is_empty());
    let owned = format!("{}-{}", PARTS.1, TAIL);
    println!("{}", owned);
}
