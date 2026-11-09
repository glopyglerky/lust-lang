// Ranges as const and static values (ts-rust bin/tsgo.rs `GO_DROPPED_RT`).
const GO_DROPPED_RT: std::ops::RangeInclusive<i32> = 35..=64;
const HALF: std::ops::Range<u8> = 2..5;
const LO: char = 'b';
static FROM: std::ops::RangeFrom<char> = LO..;
const TO: std::ops::RangeToInclusive<u64> = ..=9;
const fn lo() -> u32 {
    4
}
const R2: std::ops::RangeInclusive<u32> = lo()..=lo() * 3;
fn main() {
    for s in [3, 35, 64, 65] {
        print!("{} ", GO_DROPPED_RT.contains(&s));
    }
    println!();
    println!("{:?} {:?} {} {:?} {:?} {:?}", GO_DROPPED_RT, HALF, HALF.len(), FROM, TO, R2);
    let v: Vec<i32> = [1, 2].into_iter().chain(GO_DROPPED_RT).collect();
    println!("{} {:?} {} {}", v.len(), &v[..3], FROM.contains(&'z'), TO.contains(&10));
}
