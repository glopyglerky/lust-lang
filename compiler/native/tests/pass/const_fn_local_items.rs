// Item statements in a const evaluation: a local `const` and a local
// `const fn` in a const fn body and in a const block. A local item
// shadows a global of the same name.
const K: u32 = 1;

const fn shadow() -> u32 {
    const K: u32 = 2;
    K
}

const fn helper() -> u32 {
    const fn h() -> u32 {
        7
    }
    h() + K
}

const fn nested() -> u32 {
    let a = {
        const K: u32 = 30;
        K
    };
    a + K
}

const fn to_parts(day: u32) -> (u32, u32) {
    const S: u32 = 82;
    const L: u32 = 400 * S;
    (day / L, day % L)
}

const X: u32 = shadow();
const Y: u32 = helper();
const Z: u32 = nested();
const W: u32 = {
    const fn div_ceil(a: u64, b: u64) -> u64 {
        (a + b - 1) / b
    }
    div_ceil(1 << 32, 7) as u32
};
const P: (u32, u32) = to_parts(100_000);

fn main() {
    println!("{X} {Y} {Z} {W} {P:?} {}", shadow());
}
