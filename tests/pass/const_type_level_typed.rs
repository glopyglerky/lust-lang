// Type-level const evaluation (array lengths, const generic args,
// discriminants) runs in the operand types: u128 values past
// 2^127 stay unsigned, &mut writes in const fns land, labeled
// breaks leave the labeled loop, and shift results wrap to the
// shifted type's width.
const fn bump(x: &mut u32, by: u32) {
    *x += by;
}
const fn run(n: u32) -> u32 {
    let mut v = 0;
    let mut i = 0;
    while i < 4 {
        bump(&mut v, n);
        i += 1;
    }
    v
}
const fn lab(n: u8) -> u8 {
    let mut c = 0;
    'o: while c < 100 {
        let mut j = 0;
        while j < 10 {
            if c + j == n {
                break 'o;
            }
            j += 1;
        }
        c += 10;
    }
    c
}
fn g<const V: u128>() -> u128 {
    V
}
#[repr(u128)]
enum E {
    X = u128::MAX / 3,
}
const L: usize = ((u128::MAX >> 127) as u8) as usize;
fn main() {
    println!("{}", [0u8; run(1) as usize].len());
    println!("{}", [0u8; lab(0) as usize].len());
    println!("{}", g::<{ u128::MAX / 3 }>());
    println!("{}", E::X as u128);
    println!("{} {}", [0u8; L].len(), L);
    println!("{}", [0u8; (u128::MAX / (1u128 << 126)) as usize].len());
    println!("{}", [0u8; ((0x7fff_ffff << 2) as u64 as usize) % 5].len());
    println!("{}", [1u8; (u128::MAX >> 125) as usize].len());
}
