// Iterator::try_fold with Option and Result results (R10, ts-rust frontend/json.rs parse_hex4):
// the literal accumulator takes the closure's Try output type, the first None/Err stops the
// fold and leaves the rest in the iterator, and borrows in the accumulator are the items'.
fn hex_val(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some(u32::from(c - b'0')),
        b'a'..=b'f' => Some(u32::from(c - b'a' + 10)),
        _ => None,
    }
}

fn parse_hex4(b: &[u8]) -> Option<u32> {
    b.iter().try_fold(0, |v, &c| hex_val(c).map(|d| v * 16 + d))
}

struct Counter {
    n: u32,
}

impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.n += 1;
        if self.n < 6 { Some(self.n) } else { None }
    }
}

#[derive(Debug)]
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn sum_small(v: &[u32]) -> Result<u32, String> {
    let t = v.iter().try_fold(0u32, |a, x| if *x > 9 { Err(format!("big {x}")) } else { Ok(a + x) })?;
    Ok(t * 2)
}

fn longest<'a>(v: &'a [String], init: &'a str) -> Option<&'a str> {
    v.iter().try_fold(init, |a, s| if s.is_empty() { None } else { Some(if s.len() > a.len() { s.as_str() } else { a }) })
}

fn main() {
    println!("{:?} {:?}", parse_hex4(b"00ff"), parse_hex4(b"0g00"));
    let v = vec![1u32, 2, 30, 4];
    let mut it = v.iter();
    let r = it.try_fold(0u32, |a, x| if *x > 9 { None } else { Some(a + x) });
    println!("{:?} {:?}", r, it.next());
    println!("{:?} {:?}", sum_small(&[1, 2]), sum_small(&[3, 10, 1]));
    let mut c = Counter { n: 0 };
    println!("{:?}", c.try_fold(0u64, |a, x| a.checked_mul(10).map(|m| m + u64::from(x))));
    println!("{:?}", (1..=300).try_fold(0u8, |a, _| a.checked_add(1)));
    let names = vec![String::from("ab"), String::from("abcd"), String::from("x")];
    println!("{:?} {:?}", longest(&names, "abc"), longest(&[String::new()], "q"));
    let r: Result<D, D> = v.iter().map(|x| D(*x)).try_fold(D(0), |a, d| if d.0 > 9 { Err(d) } else { Ok(D(a.0 + d.0)) });
    println!("{:?}", r.map(|d| d.0).map_err(|d| d.0));
    let mut steps = 0;
    let r = "a1b".chars().try_fold(String::new(), |mut s, ch| {
        steps += 1;
        if ch.is_ascii_digit() {
            return Err(ch);
        }
        s.push(ch);
        Ok(s)
    });
    println!("{r:?} {steps}");
}
