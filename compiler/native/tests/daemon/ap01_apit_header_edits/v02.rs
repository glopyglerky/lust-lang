use std::fmt::Debug;

fn any_of(xs: &[u32], mut test: impl FnMut(u32) -> bool) -> bool where u32: Copy {
    xs.iter().any(|&x| test(x))
}

fn show<F: Fn(u8) -> u8>(f: F, x: impl Debug) -> String {
    format!("{:?} {}", x, f(1))
}

struct Acc(i64);

impl Acc {
    fn each(&self, f: impl Fn(i64) -> i64) -> i64 {
        f(self.0)
    }
}

fn main() {
    let n = any_of(&[1, 2], |x| x > 1);
    println!("{} {} {}", n, show(|x| x + 1, 5u8), Acc(2).each(|v| v * 2));
}
