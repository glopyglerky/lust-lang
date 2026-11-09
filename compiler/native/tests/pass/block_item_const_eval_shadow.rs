// A block enum (or const, static) that shadows a module item of the same
// name: const evaluation of its paths (const items, statics, array
// lengths, nested blocks) reads the block item, as rustc resolves it.
#![allow(dead_code)]
enum E {
    X,
    Y,
    Count,
}
const M: usize = 9;
fn main() {
    enum E {
        X = 5,
        Count = 6,
    }
    const M: usize = 7;
    let _ = (E::X, E::Count);
    let a = E::X as usize;
    const K: usize = E::X as usize;
    let t = [0u8; E::X as usize];
    let u = [1u8; E::Count as usize];
    const L: usize = M + 1;
    static Q: usize = E::Count as usize;
    let r = {
        const R: usize = E::Count as usize + M;
        R
    };
    let n = {
        enum E {
            X = 1,
        }
        const N: usize = E::X as usize;
        [0u8; E::X as usize].len() + N
    };
    println!("{a} {K} {} {} {L} {Q} {r} {n}", t.len(), u.len());
}
