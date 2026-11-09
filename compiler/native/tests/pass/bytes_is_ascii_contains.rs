// [u8]::is_ascii and str::is_ascii (lust_bytes_is_ascii: 32 bytes a step and an overlapping
// last block with SSE2, 8-byte words elsewhere) and [u8]/[i8]::contains of one byte
// (lust_memchr), at lengths 0-80 with the non-ASCII byte or the needle at each position.
fn bit(b: bool) -> char {
    if b { '1' } else { '0' }
}

fn main() {
    for n in 0..81usize {
        let base: Vec<u8> = (0..n).map(|i| b'a' + (i % 26) as u8).collect();
        let text = String::from_utf8(base.clone()).unwrap();
        let mut line = format!("{n:2} {}{} ", bit(base.is_ascii()), bit(text.is_ascii()));
        for at in 0..n {
            let mut v = base.clone();
            v[at] = if at % 2 == 0 { 0x80 } else { 0xff };
            line.push(bit(v.is_ascii()));
            line.push(bit(v[..at].is_ascii()));
        }
        line.push(' ');
        for at in 0..n {
            let mut t = String::new();
            for (i, c) in text.chars().enumerate() {
                t.push(if i == at { 'é' } else { c });
            }
            line.push(bit(t.is_ascii()));
        }
        line.push_str(&format!(" {}", bit(base.contains(&b'!'))));
        for at in 0..n {
            let mut v = base.clone();
            v[at] = b'!';
            line.push(bit(v.contains(&b'!')));
            line.push(bit(v[at + 1..].contains(&b'!')));
            let s: Vec<i8> = v.iter().map(|&b| if b == b'!' { -1 } else { b as i8 }).collect();
            line.push(bit(s.contains(&-1)));
            line.push(bit(s.as_slice().contains(&0)));
        }
        println!("{line}");
    }
    let empty: &[u8] = &[];
    println!("empty {} {} {}", empty.is_ascii(), "".is_ascii(), empty.contains(&0));
}
