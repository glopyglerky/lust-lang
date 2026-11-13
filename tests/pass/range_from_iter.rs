// RangeFrom as an iterator: next, nth (two forward steps), size_hint
// (usize::MAX, None), TrustedLen collect through take, zip, step_by,
// enumerate, skip, char steps over the surrogates, for loops.
fn main() {
    let v: Vec<u32> = (5u32..).take(3).collect();
    println!("{:?} {}", v, v.capacity());
    for (i, c) in (1..).zip("abc".chars()) {
        print!("{i}{c} ");
    }
    println!();
    let mut r = 10i8..;
    println!("{:?} {:?} {:?}", r.next(), r.nth(2), r);
    println!("{:?}", (0u64..).step_by(3).take(4).collect::<Vec<_>>());
    println!("{:?}", (1usize..).map(|x| x * x).take_while(|&x| x < 30).collect::<Vec<_>>());
    println!("{:?} {:?}", (3u8..).size_hint(), (3u8..).take(2).size_hint());
    println!("{:?} {:?}", (7i32..).position(|x| x % 5 == 0), (1u16..).find(|x| x * x > 50));
    let mut e = (200u8..).enumerate().skip(3);
    println!("{:?}", e.next());
    let mut c = '\u{D7FE}'..;
    let (x, y) = (c.next(), c.next());
    println!("{:?} {:?} {:?}", x, y, c);
    println!("{:?}", c.nth(5));
    let w: Vec<(char, i128)> = "xy".chars().zip(i128::MAX - 3..).collect();
    println!("{:?} {}", w, w.capacity());
    let mut n = 0;
    for i in 250u8.. {
        n += i as u32;
        if i == 254 {
            break;
        }
    }
    println!("{n} {:?}", (5i128..).nth(usize::MAX));
}
