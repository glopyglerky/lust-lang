// Extending borrows of drop temporaries in const items: the temporary
// lives in the item's value and is never dropped (rustc accepts all).
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct P(D, D);
struct W<'a>(&'a D, u32);
const fn mk() -> D {
    D(3)
}
const C: (D, u32) = (D(1), 5);
const NP: P = P(D(20), D(21));
const R: &D = &D(30);
const A: &u32 = &C.1;
const B: &u32 = &NP.0.0;
const E: &D = &NP.1;
static F: &u32 = &C.1;
const G: &(D, u32) = &C;
const H: &D = &mk();
const I: (&D, u32) = (&D(7), 2);
const J: W<'static> = W(&D(8), 2);
const K: &D = if true { &D(9) } else { &D(10) };
const L: &D = match 1 {
    1 => &D(11),
    _ => &D(12),
};
const M: &&D = &&D(13);
const N: [&D; 2] = [&D(14), &D(15)];
const O: &[D] = &[D(16)];
const Q: Option<&D> = Some(&D(17));
const S: &D = { const { &D(18) } };
const T: &u32 = &{ D(19) }.0;
const U: u32 = R.0;
const V: &u32 = { &C.1 };
fn main() {
    println!("{} {} {} {} {}", A, B, E.0, F, G.1);
    println!("{} {} {} {} {}", H.0, I.0.0, J.0.0, K.0, L.0);
    println!("{} {} {} {}", M.0, N[1].0, O[0].0, Q.unwrap().0);
    println!("{} {} {} {}", S.0, T, U, V);
    // A runtime field read of a const makes a copy that drops at the
    // end of its statement, as in rustc.
    let x = C.1;
    println!("x {}", x);
    println!("y {}", NP.1.0);
}
