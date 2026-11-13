// [T]::strip_prefix/strip_suffix with [T] and [T; N] patterns, and the [u8]::trim_ascii family (R13).
#[derive(Debug, PartialEq)]
struct P(i32);
fn main() {
    let s: &[u8] = b":abc\r";
    let a = s.strip_prefix(b":").unwrap_or(s);
    let b = s.strip_suffix(b"\r");
    let v: Vec<u8> = s.to_vec();
    let c = v.strip_suffix(b"\r").unwrap_or(&v);
    let d = s.strip_prefix(&[b':'][..]);
    let e = s.strip_prefix(b"x");
    let f = s.strip_suffix(&[]);
    let g = s.strip_prefix(s);
    let h = s.strip_prefix(b":abc\r!");
    println!("{a:?} {b:?} {c:?} {d:?} {e:?} {f:?} {g:?} {h:?}");
    let arr = [1, 2, 3, 4];
    println!("{:?} {:?}", arr.strip_prefix(&[1, 2]), arr.strip_suffix(&mut [3, 4]));
    let ps = vec![P(1), P(2)];
    println!("{:?}", ps.strip_prefix(&[P(1)]));
    let fl = [1.0f64, f64::NAN];
    println!("{:?} {:?}", fl.strip_prefix(&[1.0]), fl.strip_suffix(&[f64::NAN]));
    println!("{:?} {:?} {:?}", b" \t a b\x0b \n".trim_ascii_start(), b" \t a b\x0b \n".trim_ascii_end(), b"\x0c a b\r".trim_ascii());
    let w: Vec<u8> = b"  hi  ".to_vec();
    println!("{:?} {:?}", w.trim_ascii(), b"   ".trim_ascii());
}
