const fn sq(x: i32) -> i64 {
    x as i64 * x as i64 + 1
}

fn show(xs: &[i64]) {
    println!("{}", xs.len());
}

fn main() {
    let a: [u8; 9] = [0; sq(3) as usize];
    show(&[a.len() as i64]);
}
