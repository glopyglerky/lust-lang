// core::str::FromStr as a lang trait (W-STD-C phase 2): a user
// impl, a generic use through the bound, and an error type.
use std::str::FromStr;

#[derive(Debug)]
struct Pair(i32, i32);

#[derive(Debug)]
struct Bad;

fn digits(s: &str, out: &mut i32) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut v: i32 = 0;
    for c in s.chars() {
        if !c.is_ascii_digit() {
            return false;
        }
        v = v * 10 + (c as u32 - '0' as u32) as i32;
    }
    *out = v;
    true
}

impl FromStr for Pair {
    type Err = Bad;
    fn from_str(s: &str) -> Result<Pair, Bad> {
        let mut a = String::new();
        let mut b = String::new();
        let mut seen_comma = false;
        for c in s.chars() {
            if c == ',' {
                if seen_comma {
                    return Err(Bad);
                }
                seen_comma = true;
            } else if seen_comma {
                b.push(c);
            } else {
                a.push(c);
            }
        }
        if !seen_comma {
            return Err(Bad);
        }
        let mut x = 0;
        let mut y = 0;
        if digits(&a, &mut x) && digits(&b, &mut y) {
            Ok(Pair(x, y))
        } else {
            Err(Bad)
        }
    }
}

fn read<T: FromStr>(s: &str) -> Option<T> {
    T::from_str(s).ok()
}

fn main() {
    println!("{:?}", Pair::from_str("12,30"));
    println!("{:?}", Pair::from_str("12"));
    match read::<Pair>("7,8") {
        Some(p) => println!("{:?}", Some(p.0 + p.1)),
        None => println!("{:?}", None::<i32>),
    }
}
