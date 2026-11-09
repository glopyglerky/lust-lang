fn f(x: u8) -> u8 {
    match x {
        0..=127 => 1,
        128..=255 => 2,
    }
}

fn g(x: i8) -> i8 {
    match x {
        ..=-1 => -1,
        0 => 0,
        1.. => 1,
    }
}

fn h(c: char) -> u32 {
    match c {
        '\0'..='\u{D7FF}' => 1,
        '\u{E000}'..='\u{10FFFF}' => 2,
    }
}

fn u(x: usize) -> u8 {
    match x {
        0 => 0,
        1.. => 1,
    }
}

fn v(x: u32) -> u8 {
    match x {
        0..10 => 0,
        10..=20 => 1,
        21.. => 2,
    }
}

fn s(x: &[u8]) -> u32 {
    match x {
        [] => 0,
        [a] => *a as u32,
        [a, b, ..] => (*a + *b) as u32,
    }
}

fn first(x: &[u8]) -> u32 {
    match x {
        [] => 0,
        [0, ..] => 1,
        [1..=255, ..] => 2,
    }
}

fn arr(x: [u8; 2]) -> u32 {
    match x {
        [0, _] => 1,
        [1..=255, _] => 2,
    }
}

fn main() {
    println!("{} {}", f(3), f(200));
    println!("{} {} {}", g(-128), g(0), g(127));
    println!("{} {}", h('a'), h('\u{10FFFF}'));
    println!("{} {}", u(0), u(9));
    println!("{} {} {} {}", v(9), v(10), v(20), v(21));
    println!("{} {} {}", s(&[]), s(&[5]), s(&[3, 4, 9]));
    println!("{} {} {}", first(&[]), first(&[0, 1]), first(&[7]));
    println!("{} {}", arr([0, 1]), arr([3, 1]));
}
