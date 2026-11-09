// A const context in a fn body reads the fn-body item in scope, not a
// same-name item from another fn, a sibling block, or the module.
const A: u32 = 5;
const N: usize = 2;

fn first() -> u32 {
    const A: u32 = 1;
    static B: u32 = A + 10;
    B
}

fn second() -> u32 {
    const A: u32 = 2;
    static B: u32 = A + 20;
    B
}

fn main() {
    const A: u32 = 17;
    const N: usize = 3;
    static B: u32 = A * 2;
    const C: u32 = A * 3;
    let x = const { A * 2 };
    let arr = [0u8; N];
    let a = {
        const K: u32 = 1;
        const L: u32 = K + 1;
        L
    };
    let b = {
        const K: u32 = 10;
        const L: u32 = K + 1;
        L
    };
    let inner = {
        const A: u32 = 4;
        static D: u32 = A * 2;
        D
    };
    println!("{} {} {} {} {} {} {} {} {}", first(), second(), B, C, x, arr.len(), a, b, inner);
}
