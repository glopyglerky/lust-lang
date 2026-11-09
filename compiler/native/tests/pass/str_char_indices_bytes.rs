// `str::char_indices`, `str::bytes`, and `next_back` (W-STD-B): byte
// offsets of multi-byte chars from either end, and `Bytes` as std's
// `Copied<slice::Iter<u8>>` (exact size, double-ended).
fn split_at_char(s: &str, c: char) -> Option<(&str, &str)> {
    for (i, ch) in s.char_indices() {
        if ch == c {
            return Some((&s[..i], &s[i + ch.len_utf8()..]));
        }
    }
    None
}

fn main() {
    let s = "héllo wörld";
    let v: Vec<(usize, char)> = s.char_indices().collect();
    println!("{v:?} {}", v.capacity());
    println!("{:?}", s.char_indices().rev().take(3).collect::<Vec<_>>());
    let mut it = s.char_indices();
    println!("{:?} {:?} {:?} {:?}", it.next(), it.next_back(), it.next(), it.next_back());
    println!("{:?} {:?}", split_at_char(s, 'w'), split_at_char(s, 'z'));
    let b: Vec<u8> = "aé".bytes().collect();
    println!("{b:?} {} {}", b.capacity(), "xyz".bytes().rev().map(|x| x as u32).sum::<u32>());
    println!("{:?} {}", "abc".bytes().position(|x| x == b'c'), s.bytes().count());
    let arr = [1, 2, 3];
    let mut r = arr.iter();
    println!("{:?} {:?} {:?} {:?}", r.next_back(), r.next(), r.next_back(), r.next_back());
    let mut ch = "añb".chars();
    println!("{:?} {:?}", ch.next_back(), ch.next_back());
    let mut m = vec![1, 2, 3].into_iter().map(|x| x * 2);
    println!("{:?} {:?}", m.next_back(), m.next());
    let arr4 = [1, 2, 3, 4];
    let mut w = arr4.windows(3);
    println!("{:?} {:?}", w.next_back(), w.next());
}
