// capability: expressions: struct literals, with ..base update
// capability: expressions: method calls with autoderef and autoref
// capability: expressions: field access
// capability: expressions: indexing v[i] and a[i]
// capability: expressions: tuple values and tuple field access .0
// capability: expressions: array literals and [value; count] repeat
// capability: expressions: ranges (a..b, a..=b, a.., ..b, and ..)
// capability: expressions: casts with as (numeric, bool and char to integers, u8 to char, references to raw pointers)
// capability: expressions: format strings with {}, {:?}, positions, captured names, and width, fill, precision, sign, # and radix specs
// capability: expressions: vec! literals with [value; count]
// capability: expressions: assignment and compound assignment
// capability: expressions: the ? operator on Option and Result, with From error conversion
// expect: ok

#[derive(Debug, Clone, Default)]
struct Cfg {
    name: String,
    level: u32,
    on: bool,
}

#[derive(Debug)]
struct MyErr(String);

impl From<std::num::ParseIntError> for MyErr {
    fn from(e: std::num::ParseIntError) -> Self {
        MyErr(e.to_string())
    }
}

fn parse(s: &str) -> Result<u32, MyErr> {
    Ok(s.parse::<u32>()? + 1)
}

fn first_even(v: &[u32]) -> Option<u32> {
    let x = v.iter().find(|x| *x % 2 == 0)?;
    Some(*x)
}

fn main() {
    let base = Cfg { name: "b".into(), ..Default::default() };
    let c = Cfg { level: 2, ..base.clone() };
    let boxed = Box::new(String::from("abc"));
    let len = boxed.len();
    let t = (1, "two");
    let arr = [0u8; 4];
    let v = vec![7u32; 3];
    let r1: Vec<u32> = (1..4).collect();
    let r2: u32 = (1..=4).sum();
    let s = "hello";
    let parts = (&s[1..], &s[..2], &s[3..], &s[..]);
    let x = 1.9f64 as i32 + true as i32 + 'a' as i32 + (65u8 as char) as i32;
    let p = &x as *const i32;
    let mut n = 5;
    n *= 2;
    n -= 1;
    let name = "w";
    let w = 7;
    println!("{:?} {} {} {} {} {:?} {:?} {:?} {} {:?} {} {} {}", c, c.on, len, t.0, t.1, arr, v, r1, r2, parts, x, p.is_null(), n);
    println!("{1}-{0} {name}", "a", "b");
    println!("[{:>5}] [{:<4}] [{:^5}] [{:*>4}] [{:.2}] [{:+}] [{:#x}] [{:08b}] [{:o}] [{:w$}]", "r", "l", "c", 9, 3.14159, 5, 255, 5u8, 8, 3);
    println!("{:?} {:?} {:?} {:?}", parse("4"), parse("x"), first_even(&[1, 4]), first_even(&[1]));
}
