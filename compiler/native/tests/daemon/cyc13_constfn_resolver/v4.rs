const ROUTE: u8 = 0;

const fn sq(x: i16) -> i64 {
    x as i64 * x as i64
}

const N: i64 = sq(3);

fn show(xs: &[i64]) {
    println!("{}", xs.len());
}

fn main() {
    let _r = ROUTE;
    let a: [u8; 9] = [0; N as usize];
    show(&[a.len() as i64]);
}
