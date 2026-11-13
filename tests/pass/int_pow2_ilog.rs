// next_power_of_two (+checked) and ilog2/ilog10/ilog (+checked) over every width up to 64 bits (R13).
const P: usize = 33usize.next_power_of_two();
const fn lg(x: u32) -> u32 { x.ilog2() }
fn main() {
    println!("{P} {}", lg(1000));
    for x in [0u8, 1, 2, 3, 64, 65, 128, 129, 255] { println!("{x} {:?} {:?} {:?}", x.checked_next_power_of_two(), x.checked_ilog2(), x.checked_ilog10()); }
    for x in [1usize, 2, 3, 1000, usize::MAX / 2, usize::MAX / 2 + 1] { println!("{} {} {}", x.next_power_of_two(), x.ilog2(), x.ilog10()); }
    for x in [0u64, 1, u64::MAX, 1 << 63] { println!("{:?} {:?}", x.checked_next_power_of_two(), x.checked_ilog(3)); }
    for x in [i32::MIN, -1, 0, 1, 7, 8, i32::MAX] { println!("{:?} {:?} {:?} {:?}", x.checked_ilog2(), x.checked_ilog10(), x.checked_ilog(2), x.checked_ilog(-3)); }
    for (x, b) in [(81u32, 3u32), (80, 3), (1, 7), (u32::MAX, 2), (1000, 10), (5, 5), (4, 5)] { println!("{} {:?}", x.ilog(b), x.checked_ilog(b)); }
    println!("{} {}", 255u16.next_power_of_two(), 100i16.ilog(3));
    let v: Vec<u32> = [3u32, 9].iter().map(|x| x.ilog2()).collect(); println!("{v:?}");
    let v: Vec<u32> = [3u32, 5].iter().copied().map(u32::next_power_of_two).collect(); println!("{v:?}");
}
