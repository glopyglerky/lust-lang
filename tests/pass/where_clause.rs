fn get(x: i64) -> i64
where
    i64: Copy,
{
    x
}

fn main() {
    println!("{}", get(3));
}
