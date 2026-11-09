use base::LIMIT;

fn zone(x: i64) -> i64 {
    match x {
        LIMIT => 1,
        base::Gauge::RED => 2,
        0..LIMIT => 3,
        _ => 0,
    }
}

fn main() {
    println!("{} {} {} {}", zone(10), zone(90), zone(5), zone(50));
}
