// A variant with an uninhabited zero-sized payload takes no part
// in the layout: with one inhabited variant left, no tag is stored.
#[allow(dead_code)]
enum Never {}

#[allow(dead_code)]
enum E {
    A(Never),
    B(u8),
}

#[allow(dead_code)]
enum F {
    A(Never),
    B(u32),
}

#[allow(dead_code)]
enum G {
    A((u8, u32)),
    B(Never),
    C(Never),
}

fn main() {
    println!(
        "{} {} {} {}",
        std::mem::size_of::<E>(),
        std::mem::align_of::<E>(),
        std::mem::size_of::<F>(),
        std::mem::size_of::<G>()
    );
    let e = E::B(7);
    let g = G::A((1, 2));
    let x = match e {
        E::B(n) => n as u32,
        E::A(_) => 0,
    };
    let y = match g {
        G::A((a, b)) => a as u32 + b,
        _ => 0,
    };
    println!("{x} {y}");
}
