struct P {
    x: u8,
    y: u16,
}

const fn inc_u8(v: u8) -> u8 {
    v + 1
}

const T: (u8, u16) = (1, 2);
const A: [u8; 3] = [1, 2, 3];
const V: P = P { x: 4, y: 5 };
const N: u8 = inc_u8(6);
static R: &u8 = &9;
static RP: (&u8, u8) = (&7, 8);

fn main() {
    println!("{:?} {:?} {} {} {} {} {:?}", T, A, V.x, N, R, RP.0, RP);
}
