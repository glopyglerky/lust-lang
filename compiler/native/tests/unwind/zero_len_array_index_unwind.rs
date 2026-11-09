// Bounds checks on zero-sized arrays, caught: temporaries, consts and
// statics of zero-length arrays of ZST and non-ZST elements, arrays of
// ZSTs, nested arrays, slices of them, `[i]`, `&a[i]` and `get`.
use std::hint::black_box as bb;
use std::panic::catch_unwind;
#[derive(Debug, Clone, Copy)]
struct Z;
const CU: [u8; 0] = [];
const CZ: [Z; 3] = [Z, Z, Z];
static SU: [u8; 0] = [];
static SN: [[u8; 0]; 2] = [[], []];
fn main() {
    std::panic::set_hook(Box::new(|i| println!("panicked: {}", i.payload().downcast_ref::<String>().map_or("?", |s| s))));
    let i = bb(0usize);
    let j = bb(3usize);
    println!("{:?}", catch_unwind(|| [7u8; 0][i]).is_err());
    println!("{:?}", catch_unwind(|| *&[1u32; 0][i]).is_err());
    println!("{:?}", catch_unwind(|| format!("{:?}", &["a"; 0][i])).is_err());
    println!("{:?}", catch_unwind(|| [(); 0][i]).is_err());
    println!("{:?}", catch_unwind(|| [(); 3][i]).is_ok());
    println!("{:?}", catch_unwind(|| [(); 3][j]).is_err());
    println!("{:?}", catch_unwind(|| [Z; 4][j]).is_ok());
    println!("{:?}", catch_unwind(|| CU[i]).is_err());
    println!("{:?}", catch_unwind(|| CZ[j]).is_err());
    println!("{:?}", catch_unwind(|| SU[i]).is_err());
    println!("{:?}", catch_unwind(|| SN[bb(1usize)][i]).is_err());
    println!("{:?}", catch_unwind(|| [[1u8; 0]; 2][bb(1usize)][i]).is_err());
    println!("{:?}", catch_unwind(|| { let s: &[u8] = &[7u8; 0]; s[i] }).is_err());
    println!("{:?}", catch_unwind(|| [7u8; 0][i..].len()).is_ok());
    println!("{:?} {:?} {:?}", [7u8; 0].get(i), [(); 3].get(j), CZ.get(2));
}
