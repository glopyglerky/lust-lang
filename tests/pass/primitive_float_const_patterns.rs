fn main() {
    match f64::INFINITY {
        f64::INFINITY => println!("inf"),
        _ => println!("o"),
    }
    match 1.0f64 {
        f64::INFINITY => println!("inf"),
        _ => println!("o"),
    }
    match f32::NEG_INFINITY {
        f32::NEG_INFINITY => println!("ninf"),
        _ => println!("o"),
    }
    match 2.5f64 {
        <f64>::INFINITY => println!("inf"),
        _ => println!("o"),
    }
}
