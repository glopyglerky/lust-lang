// A diverging first branch (`continue`, `return`) gives the next branch no
// expected type: a nested `if` or `match` there keeps its own type.
fn digits(s: &[u8]) -> u32 {
    let mut n = 0u32;
    for &c in s {
        let d = if c == b'_' {
            n += 100;
            continue;
        } else if c.is_ascii_digit() {
            c - b'0'
        } else {
            return n + 7;
        };
        n += d as u32;
    }
    n
}

fn arms(s: &[u8]) -> u32 {
    let mut n = 0u32;
    for &c in s {
        let d = match c {
            b'_' => continue,
            b'0'..=b'9' => if c > b'5' { c - b'0' } else { 0 },
            _ => return 7,
        };
        n += d as u32;
    }
    n
}

fn main() {
    println!("{} {} {}", digits(b"1_2"), digits(b"1x"), digits(b""));
    println!("{} {}", arms(b"1_29"), arms(b"9z"));
}
