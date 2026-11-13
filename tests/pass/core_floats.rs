const PI2: f64 = 3.141592653589793 * 2.0;

fn area(r: f64) -> f64 {
    3.141592653589793 * r * r
}

fn main() {
    let x = 1.5;
    let y: f32 = 2.25;
    println!("{} {}", x, y);
    println!("{:?} {:?}", x, y);
    println!("{}", x + 0.25);
    println!("{}", x * 2.0 - 1.0);
    println!("{}", 7.5 % 2.0);
    println!("{}", area(2.0));
    println!("{}", 1e3);
    println!("{}", 1.5e-3);
    println!("{}", -x);
    println!("{} {}", x < 2.0, x == 1.5);
    println!("{}", 0.1 + 0.2);
    let nan = 0.0 / 0.0;
    println!("{} {}", nan, nan == nan);
    println!("{}", 1.0 / 0.0);
    println!("{}", -1.0 / 0.0);
    println!("{}", 2.99 as i64);
    println!("{}", -2.99 as i64);
    println!("{}", 300.7 as u8);
    println!("{}", 3i64 as f64);
    println!("{}", 3u32 as f32);
    println!("{}", 1.5f64 as f32);
    println!("{}", 16777217.0f64 as f32);
    println!("{}", (u64::MAX as f64));
    let mut acc = 0.0f64;
    acc += 1.25;
    acc *= 4.0;
    println!("{}", acc);
    println!("{:?}", 100.0f64);
    println!("{:?}", 0.5f32);
    let v = vec![1.5, 2.5];
    println!("{:?}", v);
    println!("{}", PI2);
}
