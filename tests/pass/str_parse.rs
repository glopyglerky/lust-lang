// `str::parse` over the primitive `FromStr` impls and a written
// one (tf-str): values, error kinds, Debug and Display of the
// error types, and exact float rounding.
use std::str::FromStr;

#[derive(Debug, PartialEq)]
struct Pair(i32, i32);

#[derive(Debug)]
struct Bad;

impl FromStr for Pair {
    type Err = Bad;
    fn from_str(s: &str) -> Result<Pair, Bad> {
        let (a, b) = s.split_once(',').ok_or(Bad)?;
        Ok(Pair(a.parse().map_err(|_| Bad)?, b.parse().map_err(|_| Bad)?))
    }
}

fn read<T: FromStr>(s: &str) -> Option<T> {
    s.parse().ok()
}

fn main() {
    println!("{:?} {:?}", "42".parse::<i32>(), "-42".parse::<i32>());
    println!("{:?} {:?}", "+42".parse::<u32>(), " 42".parse::<i32>());
    println!("{:?} {:?}", "".parse::<i32>(), "-".parse::<i32>());
    println!("{:?} {:?}", "2147483647".parse::<i32>(), "2147483648".parse::<i32>());
    println!("{:?} {:?}", "-2147483648".parse::<i32>(), "-2147483649".parse::<i32>());
    println!("{:?} {:?}", "999999999999999999999999x".parse::<i32>(), "12x".parse::<u8>());
    println!("{:?} {:?}", "255".parse::<u8>(), "256".parse::<u8>());
    println!("{:?} {:?}", "-128".parse::<i8>(), "-129".parse::<i8>());
    println!("{:?}", "18446744073709551615".parse::<u64>());
    println!("{:?}", "18446744073709551616".parse::<u64>());
    println!("{:?}", "340282366920938463463374607431768211455".parse::<u128>());
    println!("{:?}", "-170141183460469231731687303715884105728".parse::<i128>());
    println!("{:?} {:?}", "007".parse::<usize>(), "-0".parse::<isize>());
    println!("{:?} {:?}", "1.5".parse::<f64>(), "1e3".parse::<f32>());
    println!("{:?} {:?}", "inf".parse::<f64>(), "-Infinity".parse::<f64>());
    println!("{:?} {:?}", "NaN".parse::<f64>(), "1.5x".parse::<f64>());
    println!("{:?} {:?}", "".parse::<f64>(), ".5".parse::<f64>());
    println!("{:?} {:?}", "5.".parse::<f64>(), ".".parse::<f64>());
    println!("{:?}", "0.1".parse::<f64>().map(|v| v.to_bits()));
    println!("{:?}", "3.141592653589793238462643383279".parse::<f64>().map(|v| v.to_bits()));
    println!("{:?}", "1e-320".parse::<f64>().map(|v| v.to_bits()));
    println!("{:?} {:?}", "true".parse::<bool>(), "false".parse::<bool>());
    println!("{:?}", "True".parse::<bool>());
    println!("{:?} {:?} {:?}", "a".parse::<char>(), "é".parse::<char>(), "ab".parse::<char>());
    println!("{:?} {:?}", "".parse::<char>(), "hi".parse::<String>());
    println!("{:?} {:?}", "12,30".parse::<Pair>(), "12".parse::<Pair>());
    println!("{:?}", read::<i64>("-7"));
    let n: i32 = "42".parse().unwrap();
    let m: f64 = "2.5".parse().unwrap();
    println!("{} {}", n, m);
    let e = "x".parse::<i32>().unwrap_err();
    println!("{} {:?} {:?}", e, e, e.kind());
    println!("{}", "x".parse::<f64>().unwrap_err());
    println!("{}", "x".parse::<bool>().unwrap_err());
    println!("{}", "".parse::<char>().unwrap_err());
    println!("{:?}", "127.0.0.1".parse::<std::net::Ipv4Addr>());
}
