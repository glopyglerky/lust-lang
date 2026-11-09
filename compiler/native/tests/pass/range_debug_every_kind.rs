// Debug of every range kind as core prints it: each bound with the caller's flags and width, plain separators, " (exhausted)" for a spent RangeInclusive.
fn main() {
    println!("{:?} {:?} {:?} {:?} {:?} {:?}", 1..5, 2..=6, 3.., ..4, .., ..=7);
    println!("[{:4?}] [{:<3?}] [{:#?}] [{:+?}] [{:x?}] [{:#06X?}]", 1..5, 'a'..='c', (1u8..), ..=-2i32, 10..=255, ..300u16);
    let mut r = 1u8..=2; r.next(); r.next(); let x = r.next(); println!("{:?} {:?}", r, x);
    let mut q = 1u8..=255; for _ in 0..255 { q.next(); } println!("{:?} {:?}", q, q.is_empty());
    let rr = &&(5i64..9); println!("{:?} {rr:?} {0:>8?}|", rr);
    let s = format!("{:?}", 'x'..'z'); println!("{s} {}", s.len());
    let w = 7; println!("[{:w$?}] [{:>1$?}]", 2..3, 5);
    let mut e = 3..=3; e.next(); println!("{:?}", e);
}
