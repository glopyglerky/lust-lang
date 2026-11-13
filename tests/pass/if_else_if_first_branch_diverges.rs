// An `if` chain whose first branch diverges (a `return` statement, also
// from a macro): the other branches give the value. Also in `match`.
macro_rules! bail {
    ($v:expr) => {
        return Err($v)
    };
}

fn clamp(day: i8) -> Result<i8, u8> {
    let d = if day < 1 {
        bail!(3);
    } else if day > 28 {
        28
    } else {
        day
    };
    Ok(d)
}

fn pick(v: usize, n: usize) -> Option<usize> {
    let index = if v == 0 {
        return None;
    } else if v == n {
        if v > 100 {
            return Some(1);
        }
        v
    } else {
        v * 2
    };
    Some(index)
}

fn arm(v: u8) -> u32 {
    let x = match v {
        0 => return 99,
        1 => 10,
        _ => 20,
    };
    x + 1
}

fn main() {
    println!("{:?} {:?} {:?}", clamp(0), clamp(30), clamp(5));
    println!("{:?} {:?} {:?} {:?}", pick(0, 1), pick(3, 3), pick(200, 200), pick(4, 3));
    println!("{} {} {}", arm(0), arm(1), arm(2));
}
