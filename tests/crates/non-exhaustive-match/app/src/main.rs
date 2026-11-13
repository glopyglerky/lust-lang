fn main() {
    let k = base::Kind::A;
    let n = match k {
        base::Kind::A => 1,
        base::Kind::B(m) => m,
        base::Kind::C { y } => y,
    };
    println!("{}", n);
}
