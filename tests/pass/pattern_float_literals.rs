fn kind(x: f64) -> u32 {
    match x {
        1.5 => 1,
        2.5 | 3.5 => 2,
        4.0..=5.0 => 3,
        6.0..7.0 => 4,
        ..0.0 => 5,
        100.0.. => 6,
        _ => 0,
    }
}

const fn in_band(x: f64) -> bool {
    matches!(x, 1.0..=2.0)
}

const BAND: bool = in_band(1.5);
const OUT: bool = in_band(2.5);

fn main() {
    println!("{} {} {} {} {} {}", kind(1.5), kind(3.5), kind(4.5), kind(6.5), kind(-1.0), kind(150.0));
    println!("{} {}", BAND, OUT);
    let neg_zero: f64 = -0.0;
    println!("{} {}", matches!(neg_zero, -0.0), matches!(neg_zero, 0.0));
    let nan: f64 = f64::NAN;
    println!("{} {}", matches!(nan, 0.0..=2.0), matches!(nan, _));
    let small: f32 = 2.0;
    println!("{} {}", matches!(small, 2.0f32), matches!(small, 1.0f32..=2.0f32));
    println!("{}", matches!(1500.0f64, 1.5e3));
    let neg: f64 = -1.5;
    println!("{} {}", matches!(neg, -1.5), matches!(neg, -2.0..=-1.0));
    if let 1.5 = 1.5f64 {
        println!("iflet");
    }
    let 2.5f64 = 2.5 else {
        println!("else");
        return;
    };
    println!("letelse");
    let arr: [f64; 2] = [1.5, 2.5];
    println!("{}", matches!(arr, [1.5, _]));
    let underflow: f64 = 0.0;
    println!("{}", matches!(underflow, 1e-999));
    // Scalar float patterns auto-deref a reference scrutinee.
    let rf: &f64 = &1.5;
    println!("{} {}", matches!(rf, 1.5), matches!(rf, 1.0..=2.0));
}
