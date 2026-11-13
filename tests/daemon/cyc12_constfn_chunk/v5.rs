const fn sq(x: i64) -> i64 {
    x * x
}

fn show(xs: &[i64]) {
    println!("{:?}", xs);
}

fn main() {
    let a: [u8; 9] = [0; sq(3) as usize];
    show(&[a.len() as i64]);
}
