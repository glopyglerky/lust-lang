struct S;
impl S {
    const A: i64 = 3;
    const NAME: &str = "ess";
    fn via(x: i64) -> i64 {
        match x {
            Self::A => 1,
            _ => 0,
        }
    }
}

enum E {
    X,
    Y,
}
impl E {
    const CODE: i64 = 9;
    fn code(self) -> i64 {
        match self {
            Self::X => Self::CODE,
            Self::Y => 0,
        }
    }
}

fn main() {
    println!("{} {} {}", S::A, S::via(3), S::via(4));
    match "ess" {
        S::NAME => println!("name"),
        _ => println!("o"),
    }
    match 9 {
        E::CODE => println!("code"),
        _ => println!("o"),
    }
    println!("{}", E::X.code());
}
