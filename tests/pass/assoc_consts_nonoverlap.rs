struct G<T>(T);

impl G<i32> {
    const C: u8 = 1;
    fn via_self(&self) -> u8 { Self::C }
}
impl G<String> {
    const C: u8 = 2;
    fn via_self(&self) -> u8 { Self::C }
}

struct H<const N: usize>;
impl H<2> {
    const C: u8 = 3;
}
impl H<3> {
    const C: u8 = 4;
}

const FREE: u8 = G::<String>::C + 10;

struct Single<T>(T);
impl Single<i32> {
    const C: u8 = 5;
}

fn main() {
    println!("{} {} {}", G::<i32>::C, G::<String>::C, FREE);
    println!("{} {}", G(0i32).via_self(), G("x".to_string()).via_self());
    println!("{} {}", H::<2>::C, H::<3>::C);
    // One applicable impl needs no written argument.
    println!("{}", Single::C);
    match 2u8 {
        G::<String>::C => println!("pattern two"),
        G::<i32>::C => println!("pattern one"),
        _ => println!("pattern other"),
    }
}
