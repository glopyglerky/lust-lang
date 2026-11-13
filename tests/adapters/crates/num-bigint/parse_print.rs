// num-bigint 0.4.6: parse_bytes on signs, separators, every radix, and
// bad input; Display and Debug with format flags; to_str_radix; pow;
// ordering; and ToPrimitive (to_i64, to_u64, to_i128, to_u128, to_f64).
use num_bigint::{BigInt, Sign};
use num_traits::ToPrimitive;

fn show(b: &Option<BigInt>) -> String {
    match b {
        Some(v) => format!("{} {:?}", v, v.sign()),
        None => "None".to_string(),
    }
}

static INPUTS: [&[u8]; 22] = [
    b"0", b"-0", b"+0", b"123", b"-123", b"+123", b"-+1", b"+-1", b"++1", b"--1", b"", b"-", b"+", b"_1", b"1_000_",
    b"1__2", b"12a", b"\xff1", b"99999999999999999999999999999999", b"-18446744073709551616", b"00042", b"1 ",
];

static RADIX_INPUTS: [(&[u8], u32); 8] =
    [(b"ff", 16), (b"FF_ff", 16), (b"777", 8), (b"1011", 2), (b"zz", 36), (b"Zz", 36), (b"12", 2), (b"-g", 16)];

fn main() {
    let inputs = INPUTS;
    for s in inputs {
        println!("{}", show(&BigInt::parse_bytes(s, 10)));
    }
    for (s, r) in RADIX_INPUTS {
        println!("{}", show(&BigInt::parse_bytes(s, r)));
    }
    let big = BigInt::parse_bytes(b"-123456789012345678901234567890", 10).unwrap();
    println!("[{}] [{:>40}] [{:<36}] [{:+}] [{:?}] [{:045}]", big, big, big, big, big, big);
    let pos = BigInt::from(42i64);
    println!("[{:+}] [{:^8}] [{:08}] {}", pos, pos, pos, pos.to_str_radix(16));
    println!("{} {}", big.to_str_radix(36), BigInt::from(-255i64).to_str_radix(2));
    for (b, e) in [(2i64, 100u32), (-3, 41), (-3, 40), (0, 0), (0, 5), (10, 30), (-1, 7), (7, 0)] {
        let p = BigInt::from(b).pow(e);
        println!("{}^{} = {} bits {}", b, e, p, p.bits());
    }
    let vals = [
        BigInt::from(0i64),
        BigInt::from(-1i64),
        BigInt::from(i64::MIN),
        BigInt::from(i64::MAX),
        BigInt::from(u64::MAX),
        BigInt::parse_bytes(b"-9223372036854775809", 10).unwrap(),
        BigInt::parse_bytes(b"170141183460469231731687303715884105727", 10).unwrap(),
        BigInt::parse_bytes(b"-170141183460469231731687303715884105728", 10).unwrap(),
        BigInt::parse_bytes(b"340282366920938463463374607431768211455", 10).unwrap(),
        BigInt::parse_bytes(b"340282366920938463463374607431768211456", 10).unwrap(),
    ];
    for v in &vals {
        println!("{}: {:?} {:?} {:?} {:?} {:?} {:?} {:?}", v, v.to_i64(), v.to_u64(), v.to_i128(), v.to_u128(), v.to_i32(), v.to_u8(), v.to_f64());
    }
    println!("{}", vals[1] < vals[0] && vals[2] < vals[1] && vals[3] < vals[4] && vals[5] < vals[2]);
    for a in &vals {
        let mut line = String::new();
        for b in &vals {
            line.push(match a.cmp(b) {
                std::cmp::Ordering::Less => '<',
                std::cmp::Ordering::Equal => '=',
                std::cmp::Ordering::Greater => '>',
            });
        }
        println!("{}", line);
    }
    println!("{} {}", BigInt::from(5u32) == BigInt::from(5i32), BigInt::default());
    println!("{:?}", Sign::Minus < Sign::NoSign);
}
