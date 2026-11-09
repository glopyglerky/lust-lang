fn f(x: Option<i8>) -> u32 {
    match x {
        Some(..-3) => 0,
        Some(_) => 1,
        None => 2,
    }
}
fn main() {
    println!("{}", f(Some(-100)));
    println!("{}", f(Some(0)));
    match (5u8, true) {
        (..=9, true) => println!("lo"),
        _ => println!("hi"),
    }
    match [3u8, 20] {
        [..=9, _] => println!("slo"),
        _ => println!("shi"),
    }
    match (30u8, true) {
        (..9, true) => println!("lo"),
        _ => println!("hi"),
    }
    match Some(200u8) {
        Some(100..) => println!("big"),
        _ => println!("small"),
    }
    match (1u8, 2u8, 3u8) {
        (.., 3) => println!("rest3"),
        _ => println!("o"),
    }
    match [7u8] {
        [..] => println!("any"),
    }
}
