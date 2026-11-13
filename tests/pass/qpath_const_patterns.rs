trait Tr {
    const C: u32;
    const B: bool;
}
struct K;
impl Tr for K {
    const C: u32 = 7;
    const B: bool = true;
}
impl K {
    const A: u32 = 5;
    fn via_self(x: u32) -> u32 {
        match x {
            <Self as Tr>::C => 1,
            <Self>::A => 2,
            _ => 0,
        }
    }
}
struct S;
impl S {
    const K: u8 = 4;
}
trait Tr16 {
    const C: u16;
}
impl Tr16 for u16 {
    const C: u16 = 9;
}
#[derive(PartialEq, Eq)]
enum E {
    A,
    B,
}
trait TrE {
    const K: E;
}
impl TrE for S {
    const K: E = E::A;
}

fn f(x: u32) -> u32 {
    match x {
        <K as Tr>::C => 1,
        _ => 2,
    }
}
fn exhaustive_bool(b: bool) -> u32 {
    match b {
        <K as Tr>::B => 1,
        false => 0,
    }
}
fn sub(o: Option<u32>) -> u32 {
    match o {
        Some(<K>::A) => 1,
        Some(<K as Tr>::C | 8) => 2,
        _ => 0,
    }
}
fn main() {
    println!("{} {}", f(7), f(8));
    match 6u32 {
        <K>::A => println!("five"),
        n => println!("other {}", n),
    }
    println!("{} {} {}", K::via_self(7), K::via_self(5), K::via_self(6));
    for x in [4u8, 5] {
        match x {
            <S>::K => println!("k"),
            _ => println!("o"),
        }
    }
    for x in [9u16, 10] {
        match x {
            <u16 as Tr16>::C => println!("nine"),
            _ => println!("o"),
        }
    }
    println!("{} {}", exhaustive_bool(true), exhaustive_bool(false));
    println!("{} {} {} {}", sub(Some(5)), sub(Some(7)), sub(Some(8)), sub(Some(6)));
    let v = (5u32, true);
    match v {
        (<K>::A, true) => println!("pair"),
        _ => println!("o"),
    }
    if let <K as Tr>::C = 7u32 {
        println!("iflet");
    }
    println!("{}", matches!(8u32, <K as Tr>::C));
    match E::B {
        <E>::A => println!("a"),
        _ => println!("o"),
    }
    match E::A {
        <S as TrE>::K => println!("a"),
        _ => println!("o"),
    }
    match E::B {
        <S as TrE>::K => println!("a"),
        _ => println!("o"),
    }
    match 7u32 {
        n if n > 0 => match n {
            <K as Tr>::C => println!("guard-seven"),
            _ => println!("guard-other"),
        },
        _ => println!("o"),
    }
}
