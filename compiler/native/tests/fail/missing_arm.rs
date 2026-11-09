// expect: L0115
enum Dir {
    Up,
    Down,
    Left,
}

fn name(d: Dir) -> i32 {
    match d {
        Dir::Up => 1,
        Dir::Down => 2,
    }
}

fn main() {
    println!("{}", name(Dir::Left));
}
