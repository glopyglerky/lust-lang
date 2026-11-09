// expect: L0110
mod m {
    pub static S: i64 = 7;
}

fn main() {
    match 7 {
        m::S => println!("s"),
        _ => println!("o"),
    }
}
