// Loops with break values, continue, early return, shadowing, nested blocks.
fn find_first_square_over(limit: i64) -> i64 {
    let mut n = 0;
    loop {
        n += 1;
        if n * n > limit {
            break n * n;
        }
    }
}

fn collatz_steps(mut n: u64) -> u32 {
    let mut steps = 0;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}

fn first_negative(v: &Vec<i32>) -> i32 {
    for x in v {
        if *x < 0 {
            return *x;
        }
    }
    0
}

fn main() {
    println!("{}", find_first_square_over(50));
    println!("{}", collatz_steps(27));
    let v = vec![3, 1, -4, 1, -5];
    println!("{}", first_negative(&v));
    let x = 5;
    let x = x * 2;
    {
        let x = "inner";
        println!("{}", x);
    }
    println!("{}", x);
    let mut evens = 0;
    for i in 0..20 {
        if i % 2 == 1 {
            continue;
        }
        if i > 14 {
            break;
        }
        evens += i;
    }
    println!("{}", evens);
    let mut count = 0u8;
    for _ in 0..=255u8 {
        count = count.wrapping_add(1);
    }
    println!("{}", count);
    let big = 2i64.pow(40);
    println!("{}", big);
}
