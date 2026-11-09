const fn sq(x: i16) -> i64 {
    x as i64 * x as i64
}

fn show(xs: &[i64]) {
    println!("{}", xs.len());
}

fn main() {
    let a: [u8; 9] = [0; sq(3) as usize];
    show(&[a.len() as i64]);
}
