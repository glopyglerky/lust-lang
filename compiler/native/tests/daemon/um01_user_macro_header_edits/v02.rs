macro_rules! args {
    ($($x:expr),* $(,)?) => { vec![$($x.to_string()),*] };
}

macro_rules! tw {
    ($e:expr) => {{ let t = $e; t + t }};
}

fn chain(d: &str, n: usize) -> Vec<String> where u32: Copy {
    let mut v = args![d, n];
    v.push(String::new());
    v
}

fn twice(t: i64) -> i64 {
    tw!(t)
}

fn main() {
    println!("{:?} {}", chain("a", 2), twice(3));
}
