fn main() {
    let a: u128 = 340282366920938463463374607431768211455u128;
    println!("{}", a);
    let b: u128 = a - 1;
    println!("{}", b);
    println!("{}", b / 7);
    println!("{}", b % 7);
    let c: i128 = -170141183460469231731687303715884105728i128;
    println!("{}", c);
    println!("{}", c + 1);
    let d: u128 = 1u128 << 100;
    println!("{}", d);
    println!("{}", d >> 99);
    println!("{:?}", d);
    println!("{}", (d as u64));
    println!("{}", (258u64 as u128) * 3);
    println!("{}", a > b);
    println!("{}", a == b + 1);
    let e: i128 = -5;
    println!("{} {}", -e, e * e);
    println!("{}", u128::MAX);
    println!("{}", i128::MIN);
}
